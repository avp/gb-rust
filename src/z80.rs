use mem::Memory;
use reg::Registers;

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
