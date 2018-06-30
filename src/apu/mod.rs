use std::mem;

struct Square {
  sweep_time: u8, // 3 bit value: sweep_time/128 Hz.
  sweep_dir: i8,  // 1 if increase, -1 if decrease.
  sweep_shift: u8,

  duty_number: u8, // 2-bit index into duty table.
  length: u8,      // 5-bit length => actual length is length / 256.

  env_volume: u8, // Initial volume, between 0 and 0xf.
  env_dir: i8,    // 1 for increase, -1 for decrease.
  env_sweep: u8,  // Length of a step is env_sweep / 64.

  frequency: u16,       // Actual frequency: 131072/(2048-x) Hz.
  length_enabled: bool, // If true, stop output when length expires.
  triggered: bool,
}

impl Square {
  pub fn new() -> Square {
    Square {
      sweep_time: 0,
      sweep_dir: 1,
      sweep_shift: 0,

      duty_number: 0,
      length: 0,

      env_volume: 0,
      env_dir: 1,
      env_sweep: 0,

      frequency: 0,
      length_enabled: false,
      triggered: false,
    }
  }

  pub fn rb(&self, addr: u16) -> u8 {
    match addr {
      0 => {
        let hi = self.sweep_time;
        let mid = if self.sweep_dir == 1 { 0 } else { 1 };
        let lo = self.sweep_shift;
        (hi << 4) | (mid << 3) | lo
      }
      1 => self.duty_number,
      2 => {
        let hi = self.env_volume;
        let mid = if self.env_dir == 1 { 1 } else { 0 };
        let lo = self.env_sweep;
        (hi << 4) | (mid << 3) | lo
      }
      3 => 0xff, // Write-only register.
      4 => {
        let mid = if self.length_enabled { 1 } else { 0 };
        // Only bit 6 is readable.
        0xbf | (mid << 6)
      }
      _ => unreachable!(),
    }
  }

  pub fn wb(&mut self, addr: u16, val: u8) {
    match addr {
      0 => {
        self.sweep_time = (val >> 4) & 0x7;
        self.sweep_dir = if val & 0x08 == 0 { 1 } else { -1 };
        self.sweep_shift = val & 0x7;
      }
      1 => {
        self.length = 64 - (val & 0x3f);
      }
      2 => {
        self.env_volume = (val >> 4) & 0x7;
        self.env_dir = if val & 0x08 == 0 { -1 } else { 1 };
        self.env_sweep = val & 0x7;
      }
      3 => {
        self.frequency = (self.frequency & !0xff) | (val as u16);
      }
      4 => {
        self.frequency = (self.frequency & 0xff) | ((val as u16 & 0x7) << 8);
        self.length_enabled = val & 0x40 != 0;
        self.triggered = val & 0x80 != 0;
      }
      _ => unreachable!(),
    }
  }

  pub fn step(t: u32) {}

  pub fn next() -> i8 {
    0
  }

  /// Get the actual 8-bit duty given the duty number.
  fn duty(&self) -> u8 {
    match self.duty_number & 0b11 {
      0b00 => 0b00000001,
      0b01 => 0b10000001,
      0b10 => 0b10000111,
      0b11 => 0b01111110,
      _ => unreachable!(),
    }
  }
}

pub struct APU {
  channel1: Square,
  channel2: Square,

  sound_select: u8,

  counter: u32,
  buf: Vec<(i8, i8)>,
}

impl APU {
  pub fn new() -> APU {
    APU {
      channel1: Square::new(),
      channel2: Square::new(),

      sound_select: 0,

      counter: 0,
      buf: vec![],
    }
  }

  pub fn rb(&self, addr: u16) -> u8 {
    match addr {
      0xff10...0xff14 => self.channel1.rb(addr - 0xff10),
      0xff20...0xff24 => self.channel2.rb(addr - 0xff20),
      0xff25 => self.sound_select,
      _ => 0,
    }
  }

  pub fn wb(&mut self, addr: u16, val: u8) {
    match addr {
      0xff10...0xff14 => self.channel1.wb(addr - 0xff10, val),
      0xff20...0xff24 => self.channel2.wb(addr - 0xff20, val),
      0xff25 => self.sound_select = val,
      _ => (),
    }
  }

  pub fn step(&mut self, t: u32) {
    if t > 4 {
      self.step(4);
      self.step(t - 4);
      return;
    }

    if self.counter & ((1 << 13) & 1) == 0 {
      let idx = (self.counter >> 13) & 0x7;
      self.channel1.step(idx);
      self.channel2.step(idx);
    }

    self.counter += t;

    let (s1, s2) = (self.channel1.next(), self.channel2.next());
    let ss = self.sound_select;

    let c1 =
      if ss & 0x01 != 0 { s1 } else { 0 } + if ss & 0x02 != 0 { s2 } else { 0 };
    let c2 =
      if ss & 0x10 != 0 { s1 } else { 0 } + if ss & 0x20 != 0 { s2 } else { 0 };

    self.buf.push((c1, c2));
  }

  /// Dump the APU buffer to the caller. Clears the buffer.
  pub fn dump(&mut self) -> Vec<(i8, i8)> {
    mem::replace(&mut self.buf, vec![])
  }
}
