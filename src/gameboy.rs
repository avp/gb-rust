use cpu::CPU;
use display::Display;
use glium::glutin;
use mem::Key;
use mem::Memory;

use std::error::Error;

pub struct GameBoy {
  cpu: CPU,
  mem: Memory,
}

impl GameBoy {
  pub fn new(rom: Vec<u8>) -> Result<GameBoy, Box<Error>> {
    Ok(GameBoy {
      cpu: CPU::new(),
      mem: Memory::new(rom)?,
    })
  }

  pub fn run(&mut self, display: &mut Display) {
    let mut running = true;

    while running {
      let mut t = 0;

      debug!(
        "flags: ime={} 0b{:05b} 0b{:05b}",
        self.cpu.ime,
        self.mem.interrupt_enable,
        self.mem.interrupt_flags
      );
      // Handle interrupts.
      if self.cpu.ime && self.mem.interrupt_enable != 0 &&
        self.mem.interrupt_flags != 0
      {
        let mask = self.mem.interrupt_enable & self.mem.interrupt_flags;

        info!(
          "INTERRUPT! 0b{:05b} 0b{:05b}",
          self.mem.interrupt_enable,
          self.mem.interrupt_flags
        );
        if mask & 0b00001 != 0 {
          // VBlank
          self.mem.interrupt_flags &= !0b00001;
          t += self.cpu.handle_interrupt(&mut self.mem, 0x40);
        } else if mask & 0b00010 != 0 {
          // LCD Status
          self.mem.interrupt_flags &= !0b00010;
          t += self.cpu.handle_interrupt(&mut self.mem, 0x48);
        } else if mask & 0b00100 != 0 {
          // Timer overflow
          self.mem.interrupt_flags &= !0b00100;
          t += self.cpu.handle_interrupt(&mut self.mem, 0x50);
        } else if mask & 0b01000 != 0 {
          // Serial link
          self.mem.interrupt_flags &= !0b01000;
          t += self.cpu.handle_interrupt(&mut self.mem, 0x58);
        } else if mask & 0b10000 != 0 {
          // Joypad press
          self.mem.interrupt_flags &= !0b10000;
          t += self.cpu.handle_interrupt(&mut self.mem, 0x60);
        }
      }
      if !self.cpu.ime && self.mem.interrupt_flags != 0 {
        self.cpu.halt = false;
      }

      t += self.cpu.step(&mut self.mem);
      let ints = self.mem.step(t);
      self.mem.interrupt_flags |= ints;

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
              glutin::DeviceEvent::Key(k) => {
                if let Some(keycode) = k.virtual_keycode {
                  if let Some(key) = Key::from_code(keycode) {
                    match k.state {
                      glutin::ElementState::Pressed => {
                        self.mem.key_down(key);
                      }
                      glutin::ElementState::Released => {
                        self.mem.key_up(key);
                      }
                    }
                  }
                }
              }
              _ => (),
            }
          }
          _ => (),
        });
      }
    }
  }
}
