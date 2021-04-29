#[derive(Debug)]
struct Clock {
  main: u32,
  sub: u32,
  div: u32,
}

#[derive(Debug)]
pub struct Registers {
  pub div: u32,
  pub tima: u32,
  pub tma: u32,
  pub tac: u32,
}

#[derive(Debug)]
pub struct Timer {
  pub reg: Registers,
  clock: Clock,
}

impl Timer {
  pub fn new() -> Timer {
    Timer {
      clock: Clock {
        main: 0,
        sub: 0,
        div: 0,
      },
      reg: Registers {
        div: 0,
        tima: 0,
        tma: 0,
        tac: 0,
      },
    }
  }

  /// Updates the local registers using the m-time.
  /// Returns true if an interrupt was triggered.
  pub fn inc(&mut self, m: u32) -> bool {
    self.clock.sub += m;
    if self.clock.sub >= 4 {
      self.clock.main += 1;
      self.clock.sub -= 4;

      self.clock.div += 1;
      if self.clock.div == 16 {
        self.reg.div = (self.reg.div + 1) & 0xff;
        self.clock.div = 0;
      }
    }

    self.check_step()
  }

  /// Return true if an interrupt was triggered.
  fn check_step(&mut self) -> bool {
    if self.reg.tac & 0x4 != 0 {
      let threshold = match self.reg.tac & 3 {
        0 => 64,
        1 => 1,
        2 => 4,
        3 => 16,
        _ => panic!("Invalid & 3 result"),
      };
      if self.clock.main >= threshold {
        return self.step();
      }
    }

    false
  }

  /// Step the clocks and return true if an interrupt was triggered.
  fn step(&mut self) -> bool {
    self.clock.main = 0;
    self.reg.tima += 1;

    if self.reg.tima > 0xff {
      self.reg.tima = self.reg.tma;
      true
    } else {
      false
    }
  }
}
