use cpu::CPU;
use display::Display;
use glium::glutin;
use mem::Key;
use mem::Memory;

use std::error::Error;
use std::fmt;
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

  pub speed: Speed,
}

#[derive(Debug)]
pub enum RunError {
  SyncError,
}

impl fmt::Display for RunError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    Ok(match *self {
      RunError::SyncError => write!(f, "ROM got desynced")?,
    })
  }
}

impl Error for RunError {
  fn description(&self) -> &'static str {
    "Error running ROM"
  }
}

impl GameBoy {
  pub fn new(rom: Vec<u8>) -> Result<GameBoy, Box<Error>> {
    Ok(GameBoy {
      cpu: CPU::new(),
      mem: Memory::new(rom)?,
      speed: Speed::Normal,
    })
  }

  pub fn run(&mut self, display: &mut Display) -> Result<(), Box<Error>> {
    let mut running = true;

    let ticker = self.wait_timer(MS_PER_WAIT);

    while running {
      let clock_speed: f64 = 4.194304e+6 * self.speed.factor();
      let ticks_per_wait: u32 = (clock_speed / 1000.0 * MS_PER_WAIT as f64) as
        u32;

      // Wait a bit to catch up.
      ticker.recv()?;

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
          display.events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => {
              match event {
                glutin::WindowEvent::Closed => {
                  running = false;
                }
                _ => (),
              }
            }
            glutin::Event::DeviceEvent { event, .. } => {
              match event {
                glutin::DeviceEvent::Key(key_input) => {
                  self.handle_key(key_input);
                }
                _ => (),
              }
            }
            _ => (),
          });
        }
      }

    }
    Ok(())
  }

  pub fn title(&self) -> &str {
    str::from_utf8(&self.mem.rom[0x134..0x144]).unwrap_or("")
  }

  fn handle_key(&mut self, key_input: glutin::KeyboardInput) {
    if let Some(keycode) = key_input.virtual_keycode {
      if let Some(key) = Key::from_code(keycode) {
        match key_input.state {
          glutin::ElementState::Pressed => {
            self.mem.key_down(key);
          }
          glutin::ElementState::Released => {
            self.mem.key_up(key);
          }
        }
      }

      if let glutin::ElementState::Pressed = key_input.state {
        match keycode {
          glutin::VirtualKeyCode::S => {
            self.speed = match self.speed {
              Speed::Normal => Speed::Double,
              Speed::Double => Speed::Normal,
            };
          }
          _ => (),
        }
      }
    }
  }

  fn wait_timer(&self, ms: u32) -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || loop {
      thread::sleep(time::Duration::from_millis(ms as u64));
      if let Err(_) = tx.send(()) {
        break;
      }
    });

    rx
  }
}