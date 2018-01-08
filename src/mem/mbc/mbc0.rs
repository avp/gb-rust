use mem::mbc::MBC;

#[derive(Debug)]
pub struct MBC0 {
  rom: Vec<u8>,
  ram: Vec<u8>,
}

impl MBC0 {
  pub fn new(rom: Vec<u8>, ram: Vec<u8>) -> Self {
    Self { rom, ram }
  }
}

impl MBC for MBC0 {
  fn rb(&self, addr: u16) -> u8 {
    match addr >> 12 {
      0x0...0x7 => self.rom[addr as usize],
      0xa...0xb => self.ram[(addr & 0x1fff) as usize],
      _ => panic!("Invalid address to MBC: {}", addr),
    }
  }

  fn wb(&mut self, addr: u16, value: u8) {
    match addr >> 12 {
      0x0...0x3 => (),
      0x4...0x7 => (),
      0xa...0xb => self.ram[(addr & 0x1fff) as usize] = value,
      _ => panic!("Invalid address to MBC: {}", addr),
    }
  }

  fn eram(&self) -> &[u8] {
    &self.ram
  }
}
