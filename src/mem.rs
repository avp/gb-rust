#![cfg_attr(feature = "cargo-clippy", allow(match_same_arms))]

#[derive(Debug)]
pub struct Memory {
  bios_mapped: bool,

  bios: Vec<u8>,
  rom: Vec<u8>,
  wram: Vec<u8>,
  eram: Vec<u8>,
  zram: Vec<u8>,
}

impl Memory {
  pub fn new() -> Memory {
    Memory {
      bios_mapped: false,
      bios: vec![],
      rom: vec![],
      wram: vec![],
      eram: vec![],
      zram: vec![],
    }
  }

  /// Read a byte at address `addr`.
  pub fn rb(&mut self, addr: u16) -> u8 {
    match addr >> 12 {
      // ROM 0
      0x0...0x3 => self.rom[addr as usize],
      // ROM 1 (unbanked)
      0x4...0x7 => self.rom[addr as usize],
      // GPU VRAM
      0x8...0x9 => unimplemented!(),
      // ERAM
      0xa...0xb => self.eram[(addr & 0x1fff) as usize],
      // WRAM
      0xc...0xd => self.wram[(addr & 0x1fff) as usize],
      // WRAM Shadow
      0xe => self.wram[(addr & 0x1fff) as usize],
      0xf => {
        match (addr >> 8) & 0xf {
          // WRAM Shadow
          0x0...0xd => self.wram[(addr & 0x1fff) as usize],
          // GPU OAM
          0xe => unimplemented!(),
          0xf => {
            if addr >= 0xff80 {
              // Zero page.
              self.zram[(addr & 0x7f) as usize]
            } else {
              // I/O Control
              unimplemented!()
            }
          }
          _ => panic!("Invalid result of u16 >> 8 & 0xf"),
        }
      }
      _ => panic!("Invalid result of u16 >> 12"),
    }
  }

  /// Read a 2-byte little-endian word from `addr`.
  pub fn rw(&mut self, addr: u16) -> u16 {
    let a = u16::from(self.rb(addr));
    let b = u16::from(self.rb(addr + 1));
    (b << 8) | a
  }

  /// Write `value` at address `addr`.
  /// Panics if `addr` is in ROM.
  pub fn wb(&mut self, addr: u16, value: u8) {
    match addr >> 12 {
      // ROM 0
      0x0...0x3 => panic!("attempt to write to ROM: {}", addr),
      // ROM 1 (unbanked)
      0x4...0x7 => panic!("attempt to write to ROM: {}", addr),
      // GPU VRAM
      0x8...0x9 => unimplemented!(),
      // ERAM
      0xa...0xb => self.eram[(addr & 0x1fff) as usize] = value,
      // WRAM
      0xc...0xd => self.wram[(addr & 0x1fff) as usize] = value,
      // WRAM Shadow
      0xe => self.wram[(addr & 0x1fff) as usize] = value,
      0xf => {
        match (addr >> 8) & 0xf {
          // WRAM Shadow
          0x0...0xd => self.wram[(addr & 0x1fff) as usize] = value,
          // GPU OAM
          0xe => unimplemented!(),
          0xf => {
            if addr >= 0xff80 {
              // Zero page.
              self.zram[(addr & 0x7f) as usize] = value
            } else {
              // I/O Control
              return;
            }
          }
          _ => panic!("Invalid result of u16 >> 8 & 0xf"),
        }
      }
      _ => panic!("Invalid result of u16 >> 12"),
    }
  }

  /// Write a 2-byte little-endian word to `addr`.
  pub fn ww(&mut self, addr: u16, value: u16) {
    self.wb(addr, (value & 0xff) as u8);
    self.wb(addr + 1, ((value >> 8) & 0xff) as u8);
  }
}
