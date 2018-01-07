pub trait MBC {
  fn rb(&self, addr: u16) -> u8;
  fn wb(&mut self, addr: u16, value: u8);
}

#[derive(Debug, Copy, Clone)]
pub enum MBCMode {
  ROM,
  RAM,
}

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
      0x0...0x3 => self.rom[addr as usize],
      0x4...0x7 => self.rom[0x4000 + (addr & 0x3fff) as usize],
      0xa...0xb => self.ram[(addr & 0x1fff) as usize],
      _ => panic!("Invalid address to MBC: {}", addr),
    }
  }

  fn wb(&mut self, addr: u16, value: u8) {
    match addr >> 12 {
      0x0...0x3 => (),
      0x4...0x5 => (),
      0x4...0x7 => (),
      0xa...0xb => self.ram[(addr & 0x1fff) as usize] = value,
      _ => panic!("Invalid address to MBC: {}", addr),
    }
  }
}

#[derive(Debug)]
pub struct MBC1 {
  pub rom_bank: u8,
  pub ram_bank: u8,
  pub ram_on: bool,
  pub mode: MBCMode,
}

// impl MBC1 {
//   pub fn new() -> Self {
//     Self {
//       rom_bank: 0,
//       ram_bank: 0,
//       ram_on: false,
//       mode: MBCMode::ROM,
//     }
//   }
// }

// impl MBC for MBC1 {
//   fn read_rom(&self, addr: u16) -> u8 {
//     match addr >> 12 {
//       0x0...0x3 => self.rom[addr as usize],
//       _ => panic!("Invalid address to MBC: {}", addr);
//     }
//   }
//   fn write_rom(&mut self, addr: u16, value: u8) {}

//   fn read_ram(&self, addr: u16) -> u8 {
//     0
//   }
//   fn write_ram(&mut self, addr: u16, value: u8) {}
// }
