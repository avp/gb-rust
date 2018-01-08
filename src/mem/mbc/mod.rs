pub trait MBC {
  /// Read a byte from the MBC at `addr`.
  fn rb(&self, addr: u16) -> u8;

  /// Write `value` to the MBC at `addr`, which can update internal state.
  fn wb(&mut self, addr: u16, value: u8);

  /// Get a reference to the full cartridge RAM.
  fn eram(&self) -> &[u8];
}

mod mbc0;
pub use self::mbc0::MBC0;

mod mbc1;
pub use self::mbc1::MBC1;
