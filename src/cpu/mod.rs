mod cpu;
mod mem;
mod reg;

pub struct CPU {
  pub regs: Registers,
  pub mem: Memory,

  /// Current clock.
  m: u32,
  t: u32,

  halt: bool,
  stop: bool,
}

#[cfg(test)]
mod optest;

#[derive(Debug, Eq, PartialEq)]
pub struct Registers {
  /// General-purpose registers.
  pub a: u8,
  pub b: u8,
  pub c: u8,
  pub d: u8,
  pub e: u8,
  pub f: u8, // Flag register.
  pub h: u8,
  pub l: u8,

  /// Program counter.
  pub pc: u16,

  /// Stack pointer.
  pub sp: u16,

  /// Last Clock.
  m: u32,
  t: u32,
}

const WRAM_SIZE: usize = 8192;

pub struct Memory {
  bios_mapped: bool,

  bios: Vec<u8>,
  rom: Vec<u8>,
  wram: [u8; WRAM_SIZE],
  eram: Vec<u8>,
  zram: Vec<u8>,
}
