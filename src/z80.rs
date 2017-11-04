pub struct Z80 {
  pc: u16,
  sp: u16,

  a: u8,
  b: u8,
  c: u8,
  d: u8,
  e: u8,
  f: u8,
  h: u8,
  l: u8,
}


impl Z80 {
  pub fn new() -> Z80 {
    Z80 {
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
}

#[cfg(test)]
mod tests {
  use super::Z80;

  #[test]
  fn combine_regs() {
    let cpu = Z80::new();
    assert_eq!(cpu.af(), 0x01b0);
    assert_eq!(cpu.bc(), 0x0013);
    assert_eq!(cpu.de(), 0x00d8);
    assert_eq!(cpu.hl(), 0x014d);
  }
}
