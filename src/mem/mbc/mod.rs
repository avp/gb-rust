#[derive(Debug, Copy, Clone)]
pub enum MBCMode {
  ROM,
  RAM,
}

#[derive(Debug)]
pub struct MBC {
  pub rom_bank: u8,
  pub ram_bank: u8,
  pub ram_on: bool,
  pub mode: MBCMode,
}
