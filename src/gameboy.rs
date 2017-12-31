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

      t += self.cpu.handle_interrupt(&mut self.mem);
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
    }
  }
}
