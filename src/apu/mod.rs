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
}

impl APU {
  pub fn new() -> APU {
    APU {
      channel1: Square::new(),
      channel2: Square::new(),
    }
  }

  pub fn rb(&self, addr: u16) -> u8 {
    match addr {
      0xff10...0xff14 => self.channel1.rb(addr - 0xff10),
      0xff20...0xff24 => self.channel2.rb(addr - 0xff20),
      _ => 0,
    }
  }

  pub fn wb(&mut self, addr: u16, val: u8) {
    match addr {
      0xff10...0xff14 => self.channel1.wb(addr - 0xff10, val),
      0xff20...0xff24 => self.channel2.wb(addr - 0xff20, val),
      _ => (),
    }
  }
}
