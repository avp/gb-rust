use crate::mem::mbc::MBC;

#[derive(Debug)]
pub struct MBC3 {
  rom: Vec<u8>,
  ram: Vec<u8>,

  rom_bank: u8,
  ram_bank: u8,
  ram_on: bool,
}

impl MBC3 {
  pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
    Self {
      rom: rom,
      ram: vec![0; ram_size],

      rom_bank: 1,
      ram_bank: 0,
      ram_on: false,
    }
  }

  pub fn from_save(rom: Vec<u8>, ram: Vec<u8>) -> Self {
    Self {
      rom: rom,
      ram: ram,

      rom_bank: 1,
      ram_bank: 0,
      ram_on: false,
    }
  }

  fn rom_offset(&self) -> usize {
    self.rom_bank as usize * 0x4000
  }

  fn ram_offset(&self) -> usize {
    self.ram_bank as usize * 0x2000
  }
}

impl MBC for MBC3 {
  fn rb(&self, addr: u16) -> u8 {
    match addr >> 12 {
      0x0..=0x3 => self.rom[addr as usize],
      0x4..=0x7 => self.rom[self.rom_offset() + (addr & 0x3fff) as usize],
      0xa..=0xb => self.ram[self.ram_offset() + (addr & 0x1fff) as usize],
      _ => panic!("Invalid address to MBC: {}", addr),
    }
  }

  fn wb(&mut self, addr: u16, value: u8) {
    match addr >> 12 {
      0x0..=0x1 => self.ram_on = (value & 0x0f) == 0x0a,
      0x2..=0x3 => {
        self.rom_bank = match value & 0x7f {
          0 => 1,
          v => v,
        }
      }
      0x4...0x5 => {
        match value {
          0x0..=0x3 => self.ram_bank = value & 0x03,
          _ => {} // RTC
        }
      }
      0x6..=0x7 => {} // RTC
      0xa..=0xb => {
        let offset = self.ram_offset();
        self.ram[offset + (addr & 0x1fff) as usize] = value
      }
      _ => panic!("Invalid address to MBC: {}", addr),
    }
  }

  fn to_save(&self) -> Vec<u8> {
    self.ram.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn init() -> MBC3 {
    MBC3::new(vec![0; 0x20000], 0x20000)
  }
}
