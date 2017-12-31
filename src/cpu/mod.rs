mod cpu;
mod reg;

pub struct CPU {
  regs: Registers,

  /// Current clock.
  pub m: u32,
  t: u32,

  pub halt: bool,
  stop: bool,

  /// IME flag for global interrupt enable/disable.
  pub ime: bool,
}

#[cfg(test)]
mod optest;

#[derive(Debug, Eq, PartialEq)]
struct Registers {
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
