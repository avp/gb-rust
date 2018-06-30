use crate::cpu::CPU;
use crate::display::Display;
use crate::mem::Key;
use crate::mem::LoadError;
use crate::mem::Memory;
use crate::speaker::Speaker;

use std::error::Error;
use std::fmt;
use std::ops::Drop;
use std::path::PathBuf;
use std::str;
use std::sync::mpsc;
use std::thread;
use std::time;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Speed {
  Normal,
  Double,
}

impl Speed {
  pub fn factor(&self) -> f64 {
    match *self {
      Speed::Normal => 1.0,
      Speed::Double => 2.0,
    }
  }
}

const MS_PER_WAIT: u32 = 16;

pub struct GameBoy {
  cpu: CPU,
  mem: Memory,
  audio: Audio,

  pub speed: Speed,
  pub title: String,
}

#[derive(Debug)]
pub enum RunError {
  SyncError,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum KeyEvent {
  Pressed,
  Released,
}

impl fmt::Display for RunError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      RunError::SyncError => write!(f, "ROM got desynced")?,
    }
    Ok(())
  }
}

impl Error for RunError {
  fn description(&self) -> &'static str {
    "Error running ROM"
  }
}

impl GameBoy {
  pub fn new(rom: Vec<u8>, filename: PathBuf) -> Result<GameBoy, LoadError> {
    let title =
      String::from_utf8(rom[0x134..0x144].to_vec()).unwrap_or_default();
    Ok(GameBoy {
      title,
      cpu: CPU::new(),
      mem: Memory::new(rom, filename)?,
      audio: Audio::new().expect("Could not initialize audio"),
      speed: Speed::Normal,
    })
  }

  pub fn run(
    &mut self,
    display: Display,
    speaker: &mut Speaker,
    limit_speed: bool,
  ) {
    let mut running = true;

    let ticker = self.wait_timer(MS_PER_WAIT);

    while display.display.is_open() {
      let clock_speed: f64 = 4.194304e+6 * self.speed.factor();
      let ticks_per_wait: u32 =
        (clock_speed / 1000.0 * MS_PER_WAIT as f64) as u32;

      // Wait a bit to catch up.
      if limit_speed {
        speaker.play(self.mem.dump_audio());
        ticker.recv().unwrap();
      }

      let mut total = 0;
      while total < ticks_per_wait {
        let mut t = 0;
        t += self.cpu.handle_interrupt(&mut self.mem);
        t += self.cpu.step(&mut self.mem);
        let ints = self.mem.step(t);

        self.mem.interrupt_flags |= ints;
        total += t;

        if ints & 0b00001 != 0 {
          display.redraw(self.mem.frame());
          if let Some(keys) =
            display.display.get_keys_pressed(minifb::KeyRepeat::No)
          {
            for key in &keys {
              self.handle_key(*key, KeyEvent::Pressed);
            }
          }
          if let Some(keys) = display.display.get_keys_released() {
            for key in &keys {
              self.handle_key(*key, KeyEvent::Released);
            }
          }
        }
      }
    }
  }

  fn handle_key(&mut self, key: minifb::Key, event: KeyEvent) {
    if let Some(key) = Key::from_code(key) {
      match event {
        KeyEvent::Pressed => {
          self.mem.key_down(key);
        }
        KeyEvent::Released => {
          self.mem.key_up(key);
        }
      }
    }

    if let KeyEvent::Pressed = event {
      if let minifb::Key::S = key {
        self.speed = match self.speed {
          Speed::Normal => Speed::Double,
          Speed::Double => Speed::Normal,
        };
        println!("Speed set to: {}", self.speed.factor());
      }
    }
  }

  fn wait_timer(&self, ms: u32) -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || loop {
      thread::sleep(time::Duration::from_millis(ms as u64));
      if tx.send(()).is_err() {
        break;
      }
    });

    rx
  }
}

impl Drop for GameBoy {
  fn drop(&mut self) {
    self.mem.save_ram();
  }
}
