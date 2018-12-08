use crate::cpu::Registers;

pub const Z: u8 = 0x80;
pub const N: u8 = 0x40;
pub const H: u8 = 0x20;
pub const C: u8 = 0x10;

impl Registers {
  pub fn new() -> Registers {
    Registers {
      a: 0x01,
      f: 0xb0,
      b: 0x00,
      c: 0x13,
      d: 0x00,
      e: 0xd8,
      h: 0x01,
      l: 0x4d,

      sp: 0xfffe,
      pc: 0x100,

      m: 0,
      t: 0,
    }
  }

  pub fn af(&self) -> u16 {
    (u16::from(self.a) << 8) | u16::from(self.f)
  }
  pub fn bc(&self) -> u16 {
    (u16::from(self.b) << 8) | u16::from(self.c)
  }
  pub fn de(&self) -> u16 {
    (u16::from(self.d) << 8) | u16::from(self.e)
  }
  pub fn hl(&self) -> u16 {
    (u16::from(self.h) << 8) | u16::from(self.l)
  }

  pub fn hl_inc(&mut self) {
    if self.l == 0xff {
      self.h = self.h.wrapping_add(1);
    }
    self.l = self.l.wrapping_add(1);
  }
  pub fn hl_dec(&mut self) {
    if self.l == 0 {
      self.h = self.h.wrapping_sub(1);
    }
    self.l = self.l.wrapping_sub(1);
  }

  pub fn z(&self) -> bool {
    //! Zero flag
    self.f & Z != 0
  }
  pub fn h(&self) -> bool {
    //! Subtract flag
    self.f & H != 0
  }
  pub fn n(&self) -> bool {
    //! Half carry flag
    self.f & N != 0
  }
  pub fn c(&self) -> bool {
    //! Carry flag
    self.f & C != 0
  }
}

#[cfg(test)]
mod tests {
  use super::Registers;

  #[test]
  fn combine_regs() {
    let regs = Registers::new();
    assert_eq!(regs.af(), 0x01b0);
    assert_eq!(regs.bc(), 0x0013);
    assert_eq!(regs.de(), 0x00d8);
    assert_eq!(regs.hl(), 0x014d);
    assert_eq!(regs.pc, 0x100);
    assert_eq!(regs.sp, 0xfffe);
  }

  #[test]
  fn flags() {
    let regs = Registers::new();
    assert!(regs.z());
    assert!(!regs.n());
    assert!(regs.h());
    assert!(regs.c());
  }
}
