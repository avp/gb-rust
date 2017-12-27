#![cfg_attr(feature = "cargo-clippy", allow(match_same_arms))]

mod bios;

use gpu;

const WRAM_SIZE: usize = 0x2000;
const ERAM_SIZE: usize = 0x2000;
const ZRAM_SIZE: usize = 0xff;

pub struct Memory {
  bios: Vec<u8>,
  rom: Vec<u8>,
  wram: Vec<u8>,
  eram: Vec<u8>,
  zram: Vec<u8>,

  pub gpu: Box<gpu::GPU>,
}

impl Memory {
  pub fn new(rom: Vec<u8>) -> Memory {
    Memory {
      bios: bios::BIOS.to_vec(),
      rom: rom,
      wram: vec![0; WRAM_SIZE],
      eram: vec![0; ERAM_SIZE],
      zram: vec![0; ZRAM_SIZE],

      gpu: Box::new(gpu::GPU::new()),
    }
  }

  /// Read a byte at address `addr`.
  pub fn rb(&self, addr: u16) -> u8 {
    match addr >> 12 {
      // ROM 0
      0x0...0x3 => self.rom[addr as usize],
      // ROM 1 (unbanked)
      0x4...0x7 => self.rom[addr as usize],
      // GPU VRAM
      0x8...0x9 => self.gpu.vram[(addr & 0x1fff) as usize],
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
          0xe => {
            let idx = (addr & 0xff) as usize;
            if idx < gpu::OAM_SIZE {
              self.gpu.oam[idx]
            } else {
              0
            }
          }
          0xf => {
            if addr >= 0xff80 {
              // Zero page.
              self.zram[(addr & 0x7f) as usize]
            } else {
              // I/O Control
              match (addr >> 4) & 0xf {
                0x4...0x7 => self.gpu.rb(addr),
                _ => 0,
              }
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
    // debug!("MMU: 0x{:x} <- 0x{:x}", addr, value);
    if addr == 0xff02 && value == 0x81 {
      print!("{}", self.rb(0xff01) as char);
    }
    match addr >> 12 {
      // ROM 0
      0x0...0x3 => (),
      // ROM 1 (unbanked)
      0x4...0x7 => (),
      // GPU VRAM
      0x8...0x9 => {
        self.gpu.vram[(addr & 0x1fff) as usize] = value;
        self.gpu.update_tile(addr);
        println!("WRITE VRAM: 0x{:x} <- 0x{:x}", addr, value);
        println!("VRAM = {:?}", &self.gpu.vram[0x1800..0x1830]);
      }
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
          0xe => {
            let idx = (addr & 0xff) as usize;
            if idx < gpu::OAM_SIZE {
              self.gpu.oam[idx] = value;
              // TODO: Call self.gpu.updateoam(addr, val)
            }
          }
          0xf => {
            if addr >= 0xff80 {
              // Zero page.
              self.zram[(addr & 0x7f) as usize] = value
            } else {
              // I/O Control
              match (addr >> 4) & 0xf {
                0x4...0x7 => self.gpu.wb(addr, value),
                _ => (),
              }
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

  /// Write an arbitrary number of bytes to memory.
  pub fn write(&mut self, addr: u16, values: &[u8]) {
    let mut cur = addr;
    for v in values {
      self.wb(cur, *v);
      cur += 1;
    }
  }
}
