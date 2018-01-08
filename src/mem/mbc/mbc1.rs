use mem::mbc::MBC;
use mem::mbc::MBCMode;

#[derive(Debug)]
pub struct MBC1 {
  rom: Vec<u8>,
  ram: Vec<u8>,

  rom_bank: u8,
  ram_bank: u8,
  ram_on: bool,
  mode: MBCMode,
}

impl MBC1 {
  pub fn new(rom: Vec<u8>, ram: Vec<u8>) -> Self {
    Self {
      rom: rom,
      ram: ram,

      rom_bank: 1,
      ram_bank: 0,
      ram_on: false,
      mode: MBCMode::ROM,
    }
  }

  fn rom_offset(&self) -> usize {
    self.rom_bank as usize * 0x4000
  }

  fn ram_offset(&self) -> usize {
    self.ram_bank as usize * 0x2000
  }
}

impl MBC for MBC1 {
  fn rb(&self, addr: u16) -> u8 {
    match addr >> 12 {
      0x0...0x3 => self.rom[addr as usize],
      0x4...0x7 => self.rom[self.rom_offset() + (addr & 0x3fff) as usize],
      0xa...0xb => self.ram[self.ram_offset() + (addr & 0x1fff) as usize],
      _ => panic!("Invalid address to MBC: {}", addr),
    }
  }

  fn wb(&mut self, addr: u16, value: u8) {
    match addr >> 12 {
      0x0...0x1 => self.ram_on = (value & 0x0f) == 0x0a,
      0x2...0x3 => {
        self.rom_bank = (self.rom_bank & 0x60) +
          match value & 0x1f {
            0 => 1,
            v => v,
          }
      }
      0x4...0x5 => {
        match self.mode {
          MBCMode::RAM => self.ram_bank = value & 0x03,
          MBCMode::ROM => {
            self.rom_bank = (self.rom_bank & 0x1f) + ((value & 0x03) << 5)
          }
        }
      }
      0xa...0xb => {
        let offset = self.ram_offset();
        self.ram[offset + (addr & 0x1fff) as usize] = value
      }
      _ => panic!("Invalid address to MBC: {}", addr),
    }
  }

  fn eram(&self) -> &[u8] {
    &self.ram
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn init() -> MBC1 {
    MBC1::new(vec![0; 0x20000], vec![0; 0x2000])
  }

  #[test]
  fn default_bank() {
    let mut mbc = init();
    mbc.rom[0] = 1;
    assert_eq!(mbc.rb(0), 1);
    mbc.rom[0x4000] = 2;
    assert_eq!(mbc.rb(0x4000), 2);
  }
}