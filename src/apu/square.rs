pub struct Square {
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

  duty: u8, // Actual duty at current step, refreshed on trigger.
  period_count: u32, // Period count, starts at 2048 - frequency.
  enabled: bool, // Square wave volume enabled?
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

      duty: get_duty(0),
      period_count: 2048,
      enabled: false,
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
        self.duty_number = val >> 6;
      }
      2 => {
        self.env_volume = (val >> 4) & 0x7;
        self.env_dir = if val & 0x08 == 0 { -1 } else { 1 };
        self.env_sweep = val & 0x7;
      }
      3 => {
        self.frequency = (self.frequency & !0xffu16) | (val as u16);
        info!("FREQ={}", self.frequency);
      }
      4 => {
        self.frequency = (self.frequency & 0xffu16) | ((val as u16 & 0x7) << 8);
        self.length_enabled = val & 0x40 != 0;
        self.triggered = val & 0x80 != 0;
        if val & 0x80 != 0 {
          // eprintln!("TRIGGERED");
        }
        info!("freq={}", self.frequency);
      }
      _ => unreachable!(),
    }
  }

  /// Tick the internal frame sequencer by the 3-bit value `idx`.
  pub fn step(&mut self, idx: u32) {
    if idx & 1 == 0 && self.length_enabled && self.length > 0 {
      self.length -= 1;
      if self.length == 0 {
        self.enabled = false;
      }
    }
  }

  fn reset_period(&mut self) {
    self.period_count = (2048 - (self.frequency as u32)) * 4;
  }

  pub fn next(&mut self) -> i8 {
    if self.triggered {
      self.duty = get_duty(self.duty_number);
      self.reset_period();
      self.triggered = false;
      self.enabled = true;
      self.length = 64;
    }

    if !self.enabled {
      return 0;
    }

    if self.period_count == 0 {
      self.reset_period();
      self.duty = self.duty.rotate_left(1);
    }
    // eprintln!("duty=0b{:08b}", self.duty);

    eprintln!("volume={}", self.env_volume);
    eprintln!("env_sweep={}", self.env_sweep);
    let volume: i8 = 100;

    let result = if self.duty & 0x80 != 0 {
      volume
    } else {
      -volume
    };

    info!("period count = {}", self.period_count);
    self.period_count -= 1;

    result
  }
}

/// Get the actual 8-bit duty given the duty number.
fn get_duty(duty_number: u8) -> u8 {
  match duty_number & 0b11 {
    0b00 => 0b00000001,
    0b01 => 0b10000001,
    0b10 => 0b10000111,
    0b11 => 0b01111110,
    _ => unreachable!(),
  }
}
