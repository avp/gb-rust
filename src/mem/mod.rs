#![cfg_attr(feature = "cargo-clippy", allow(match_same_arms))]

mod key;
mod timer;

pub use self::key::Key;
use self::key::KeyData;
use gpu;

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::stdout;
use std::path::PathBuf;

const WRAM_SIZE: usize = 0x2000;
const ERAM_SIZE: usize = 0x2000;
const ZRAM_SIZE: usize = 0xff;

#[derive(Debug)]
enum CartridgeType {
  NoMBC,
  MBC1,
  MBC1RAM,
  MBC1BatteryRAM,
}

impl fmt::Display for CartridgeType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      CartridgeType::NoMBC => write!(f, "No MBC")?,
      CartridgeType::MBC1 => write!(f, "MBC1")?,
      CartridgeType::MBC1RAM => write!(f, "MBC1 with RAM")?,
      CartridgeType::MBC1BatteryRAM => {
        write!(f, "MBC1 with battery-backed RAM")?
      }
    }
    Ok(())
  }
}

impl CartridgeType {
  pub fn has_battery(&self) -> bool {
    match *self {
      CartridgeType::MBC1BatteryRAM => true,
      _ => false,
    }
  }
}

#[derive(Debug)]
enum MBCMode {
  ROM,
  RAM,
}

#[derive(Debug)]
struct MBC {
  rom_bank: u8,
  ram_bank: u8,
  ram_on: bool,
  mode: MBCMode,
}

#[derive(Debug)]
pub enum LoadError {
  InvalidROM,
  InvalidCartridgeType(u8),
}

impl fmt::Display for LoadError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}: ", self.description())?;
    match *self {
      LoadError::InvalidROM => write!(f, "Invalid ROM")?,
      LoadError::InvalidCartridgeType(t) => {
        write!(f, "Invalid cartridge type (0x{:02x})", t)?
      }
    };
    Ok(())
  }
}

impl Error for LoadError {
  fn description(&self) -> &str {
    "Error loading ROM"
  }
}

pub struct Memory {
  pub rom: Vec<u8>,
  wram: Vec<u8>,
  eram: Vec<u8>,
  zram: Vec<u8>,
  key: KeyData,

  sb: u8,
  sc: u8,

  mbc1: MBC,
  rom_offset: usize,
  ram_offset: usize,
  cartridge_type: CartridgeType,

  pub interrupt_enable: u8,
  pub interrupt_flags: u8,

  gpu: Box<gpu::GPU>,
  timer: Box<timer::Timer>,

  savepath: PathBuf,
}

impl Memory {
  pub fn new(rom: Vec<u8>, filename: &PathBuf) -> Result<Memory, LoadError> {
    let cartridge_type = match rom.get(0x0147) {
      Some(&t) => {
        match t {
          0 => CartridgeType::NoMBC,
          1 => CartridgeType::MBC1,
          2 => CartridgeType::MBC1RAM,
          3 => CartridgeType::MBC1BatteryRAM,
          t => return Err(LoadError::InvalidCartridgeType(t)),
        }
      }
      None => return Err(LoadError::InvalidROM),
    };
    info!("Loading cartridge: {}", cartridge_type);

    let mut eram: Vec<u8> = vec![];
    let savepath = filename.with_extension("sav");
    if cartridge_type.has_battery() && savepath.is_file() {
      // Load ERAM from save file.
      if let Ok(mut f) = File::open(&savepath) {
        println!("Reading save file: {}", savepath.to_str().unwrap());
        match f.read_to_end(&mut eram) {
          Ok(_) => (),
          Err(_) => {
            eprintln!(
              "Unable to read save file: {}",
              savepath.to_str().unwrap()
            );
          }
        }
      }
    }
    // Error correction
    if eram.len() != ERAM_SIZE {
      eram = vec![0; ERAM_SIZE]
    }

    let mut result = Memory {
      rom: rom,
      wram: vec![0; WRAM_SIZE],
      eram: eram,
      zram: vec![0; ZRAM_SIZE],
      key: KeyData::new(),

      sb: 0,
      sc: 0,

      mbc1: MBC {
        rom_bank: 0,
        ram_bank: 0,
        ram_on: false,
        mode: MBCMode::ROM,
      },
      rom_offset: 0x4000,
      ram_offset: 0x0000,
      cartridge_type: cartridge_type,

      interrupt_enable: 0,
      interrupt_flags: 0,

      gpu: Box::new(gpu::GPU::new()),
      timer: Box::new(timer::Timer::new()),

      savepath: savepath,
    };
    result.power_on();

    Ok(result)
  }

  fn power_on(&mut self) {
    // See http://nocash.emubase.de/pandocs.htm#powerupsequence
    self.wb(0xff05, 0x00); // TIMA
    self.wb(0xff06, 0x00); // TMA
    self.wb(0xff07, 0x00); // TAC
    self.wb(0xff10, 0x80); // NR10
    self.wb(0xff11, 0xbf); // NR11
    self.wb(0xff12, 0xf3); // NR12
    self.wb(0xff14, 0xbf); // NR14
    self.wb(0xff16, 0x3f); // NR21
    self.wb(0xff17, 0x00); // NR22
    self.wb(0xff19, 0xbf); // NR24
    self.wb(0xff1a, 0x7f); // NR30
    self.wb(0xff1b, 0xff); // NR31
    self.wb(0xff1c, 0x9F); // NR32
    self.wb(0xff1e, 0xbf); // NR33
    self.wb(0xff20, 0xff); // NR41
    self.wb(0xff21, 0x00); // NR42
    self.wb(0xff22, 0x00); // NR43
    self.wb(0xff23, 0xbf); // NR30
    self.wb(0xff24, 0x77); // NR50
    self.wb(0xff25, 0xf3); // NR51
    self.wb(0xff26, 0xf1); // NR52
    self.wb(0xff40, 0xb1); // LCDC, tweaked to turn the window on
    self.wb(0xff42, 0x00); // SCY
    self.wb(0xff43, 0x00); // SCX
    self.wb(0xff45, 0x00); // LYC
    self.wb(0xff47, 0xfc); // BGP
    self.wb(0xff48, 0xff); // OBP0
    self.wb(0xff49, 0xff); // OBP1
    self.wb(0xff4a, 0x00); // WY
    self.wb(0xff4b, 0x07); // WX, tweaked to position the window at (0, 0)
    self.wb(0xffff, 0x00); // IE
  }

  /// Steps the MMU by t t-time.
  /// Returns the interrupts that have fired.
  pub fn step(&mut self, t: u32) -> u8 {
    let mut int = 0;
    int |= self.gpu.step(t);

    let m = if t == 0 { 1 } else { t / 4 };
    if self.timer.inc(m) {
      int |= 0b00100;
    };
    int
  }

  /// Return a reference to the current frame to draw.
  pub fn frame(&self) -> &gpu::Frame {
    &self.gpu.frame
  }

  /// Read a byte at address `addr`.
  pub fn rb(&self, addr: u16) -> u8 {
    match addr >> 12 {
      // ROM 0
      0x0...0x3 => self.rom[addr as usize],
      // ROM 1
      0x4...0x7 => self.rom[self.rom_offset + (addr & 0x3fff) as usize],
      // GPU VRAM
      0x8...0x9 => self.gpu.vram[(addr & 0x1fff) as usize],
      // ERAM
      0xa...0xb => self.eram[self.ram_offset + (addr & 0x1fff) as usize],
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
            if addr == 0xffff {
              self.interrupt_enable
            } else if addr >= 0xff80 {
              // Zero page.
              self.zram[(addr & 0x7f) as usize]
            } else if addr >= 0xff40 {
              // I/O Control
              match (addr >> 4) & 0xf {
                0x4...0x7 => self.gpu.rb(addr),
                _ => 0,
              }
            } else {
              match addr & 0x3f {
                0x00 => self.key.rb(),
                0x01 => self.sb,
                0x02 => self.sc,
                0x04 => self.timer.reg.div as u8,
                0x05 => self.timer.reg.tima as u8,
                0x06 => self.timer.reg.tma as u8,
                0x07 => self.timer.reg.tac as u8,
                0x0f => self.interrupt_flags,
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
  pub fn wb(&mut self, addr: u16, value: u8) {
    if addr == 0xff02 && value == 0x81 {
      print!("{}", self.rb(0xff01) as char);
      stdout().flush().unwrap();
    }
    match addr >> 12 {
      // ROM 0
      0x0...0x1 => {
        match self.cartridge_type {
          CartridgeType::MBC1RAM |
          CartridgeType::MBC1BatteryRAM => {
            self.mbc1.ram_on = (value & 0x0f) == 0x0a;
          }
          _ => (),
        }
      }
      0x2...0x3 => {
        match self.cartridge_type {
          CartridgeType::MBC1 |
          CartridgeType::MBC1RAM |
          CartridgeType::MBC1BatteryRAM => {
            let value = value & 0x1f;
            let value = if value == 0 { 1 } else { value };
            self.mbc1.rom_bank = (self.mbc1.rom_bank & 0x60) + value;
            self.rom_offset = self.mbc1.rom_bank as usize * 0x4000;
          }
          _ => (),
        }
      }
      0x4...0x5 => {
        match self.cartridge_type {
          CartridgeType::MBC1 |
          CartridgeType::MBC1RAM |
          CartridgeType::MBC1BatteryRAM => {
            match self.mbc1.mode {
              MBCMode::RAM => {
                self.mbc1.ram_bank = value & 0x03;
                self.ram_offset = self.mbc1.ram_bank as usize * 0x2000;
              }
              MBCMode::ROM => {
                self.mbc1.rom_bank = (self.mbc1.rom_bank & 0x1f) +
                  ((value & 0x03) << 5);
                self.rom_offset = self.mbc1.rom_bank as usize * 0x4000;
              }
            }
          }
          _ => (),
        }
      }
      // ROM 1 (unbanked)
      0x6...0x7 => {
        match self.cartridge_type {
          CartridgeType::MBC1RAM |
          CartridgeType::MBC1BatteryRAM => {
            self.mbc1.mode = if value & 1 == 1 {
              MBCMode::RAM
            } else {
              MBCMode::ROM
            };
          }
          _ => (),
        }
      }
      // GPU VRAM
      0x8...0x9 => {
        debug!("VRAM: 0x{:04x} <- 0x{:02x}", addr, value);
        self.gpu.vram[(addr & 0x1fff) as usize] = value;
        self.gpu.update_tile(addr);
      }
      // ERAM
      0xa...0xb => {
        self.eram[self.ram_offset + (addr & 0x1fff) as usize] = value
      }
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
            debug!("OAM: 0x{:02x} <- {}", idx, value);
            if idx < gpu::OAM_SIZE {
              self.gpu.oam[idx] = value;
              self.gpu.update_object(addr, value);
            }
          }
          0xf => {
            if addr == 0xffff {
              self.interrupt_enable = value;
            } else if addr >= 0xff80 {
              // Zero page.
              self.zram[(addr & 0x7f) as usize] = value
            } else if addr >= 0xff40 {
              // OAM DMA?
              if addr == 0xff46 {
                for i in 0..160 {
                  let v = self.rb(((value as u16) << 8) + i);
                  self.wb(0xfe00 + i, v);
                }
                return;
              }

              match (addr >> 4) & 0xf {
                0x4...0x7 => self.gpu.wb(addr, value),
                _ => (),
              }
            } else {
              match addr & 0x3f {
                0x00 => self.key.wb(value),
                0x01 => self.sb = value,
                0x02 => self.sc = value,
                0x04 => self.timer.reg.div = value as u32,
                0x05 => self.timer.reg.tima = value as u32,
                0x06 => self.timer.reg.tma = value as u32,
                0x07 => self.timer.reg.tac = value as u32,
                0x0f => self.interrupt_flags = value,
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

  pub fn key_down(&mut self, key: Key) {
    self.interrupt_flags |= 0b10000;
    self.key.key_down(key);
  }

  pub fn key_up(&mut self, key: Key) {
    self.interrupt_flags |= 0b10000;
    self.key.key_up(key);
  }

  pub fn save_ram(&self) {
    let savepath = &self.savepath;
    if self.cartridge_type.has_battery() && !savepath.is_dir() {
      // Load ERAM from save file.
      match File::create(&savepath) {
        Ok(mut f) => {
          println!("Writing save file: {}", savepath.to_str().unwrap());
          match f.write(&self.eram) {
            Ok(_) => (),
            Err(e) => {
              eprintln!(
                "Unable to write save file {}: {}",
                savepath.to_str().unwrap(),
                e
              );
            }
          }
        }
        Err(e) => {
          eprintln!(
            "Unable to open save file {}: {}",
            savepath.to_str().unwrap(),
            e
          );
        }
      }
    }

  }
}
