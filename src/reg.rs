#[derive(Debug, Eq, PartialEq)]
pub struct Registers {
  /// General-purpose registers.
  a: u8,
  b: u8,
  c: u8,
  d: u8,
  e: u8,
  f: u8, // Flag register.
  h: u8,
  l: u8,

  /// Program counter.
  pc: u16,

  /// Stack pointer.
  sp: u16,

  /// Last Clock.
  m: u32,
  t: u32,
}

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
    ((self.a as u16) << 8) | (self.f as u16)
  }
  pub fn bc(&self) -> u16 {
    ((self.b as u16) << 8) | (self.c as u16)
  }
  pub fn de(&self) -> u16 {
    ((self.d as u16) << 8) | (self.e as u16)
  }
  pub fn hl(&self) -> u16 {
    ((self.h as u16) << 8) | (self.l as u16)
  }

  pub fn z(&self) -> bool {
    //! Zero flag
    ((self.f >> 7) & 1) == 1
  }
  pub fn h(&self) -> bool {
    //! Subtract flag
    ((self.f >> 6) & 1) == 1
  }
  pub fn n(&self) -> bool {
    //! Half carry flag
    ((self.f >> 5) & 1) == 1
  }
  pub fn c(&self) -> bool {
    //! Carry flag
    ((self.f >> 4) & 1) == 1
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
    assert!(!regs.h());
    assert!(regs.n());
    assert!(regs.c());
  }
}
