use mem::Memory;

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

#[derive(Debug)]
pub struct Z80 {
  pub regs: Registers,
  pub mem: Memory,

  /// Current clock.
  m: u32,
  t: u32,
}


impl Z80 {
  pub fn new() -> Z80 {
    Z80 {
      regs: Registers::new(),
      mem: Memory::new(),
      m: 0,
      t: 0,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Z80;

  #[test]
  fn combine_regs() {
    let cpu = Z80::new();
    assert_eq!(cpu.regs.af(), 0x01b0);
    assert_eq!(cpu.regs.bc(), 0x0013);
    assert_eq!(cpu.regs.de(), 0x00d8);
    assert_eq!(cpu.regs.hl(), 0x014d);
    assert_eq!(cpu.regs.pc, 0x100);
    assert_eq!(cpu.regs.sp, 0xfffe);
  }

  #[test]
  fn flags() {
    let cpu = Z80::new();
    assert!(cpu.regs.z());
    assert!(!cpu.regs.h());
    assert!(cpu.regs.n());
    assert!(cpu.regs.c());
  }
}
