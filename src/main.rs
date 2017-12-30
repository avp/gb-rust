#![allow(dead_code)]
#![cfg_attr(test, allow(dead_code))]

extern crate clap;
use clap::{App, Arg};

extern crate glium;
use glium::glutin;

#[macro_use]
extern crate log;
extern crate env_logger;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Read;
use std::process;

mod cpu;
mod mem;
mod gpu;
mod display;

struct Args {
  rom: String,
}

fn main() {
  match main_result() {
    Ok(_) => (),
    Err(e) => {
      eprintln!("{}", e);
      process::exit(1);
    }
  }
}

fn main_result() -> Result<(), Box<Error>> {
  env_logger::init().unwrap();

  let args = get_args()?;

  let mut cpu = cpu::CPU::new();

  info!("Reading ROM: {}", &args.rom);
  let rom = read_file(&args.rom)?;
  info!("Loading ROM...");
  let mut mem = mem::Memory::new(rom)?;
  info!("ROM loaded.");

  let mut display = display::Display::new();

  run(&mut cpu, &mut mem, &mut display);
  Ok(())
}

fn get_args() -> Result<Args, Box<Error>> {
  let matches = App::new("GB Rust")
    .version("0.1.0")
    .author("AVP <avp@avp42.com>")
    .about("Game Boy emulator")
    .arg(
      Arg::with_name("rom")
        .required(true)
        .help("Path to the Game Boy ROM file to load")
        .value_name("FILE"),
    )
    .get_matches();

  let rom = matches.value_of("rom").unwrap();
  Ok(Args { rom: String::from(rom) })
}

fn read_file(filename: &str) -> Result<Vec<u8>, io::Error> {
  let mut file = File::open(filename)?;
  let mut result: Vec<u8> = vec![];
  file.read_to_end(&mut result)?;
  Ok(result)
}

fn run(
  cpu: &mut cpu::CPU,
  mem: &mut mem::Memory,
  display: &mut display::Display,
) {
  let mut running = true;

  while running {
    let mut t = 0;

    debug!(
      "flags: ime={} 0b{:05b} 0b{:05b}",
      cpu.ime,
      mem.interrupt_enable,
      mem.interrupt_flags
    );
    // Handle interrupts.
    if cpu.ime && mem.interrupt_enable != 0 && mem.interrupt_flags != 0 {
      let mask = mem.interrupt_enable & mem.interrupt_flags;

      info!(
        "INTERRUPT! 0b{:05b} 0b{:05b}",
        mem.interrupt_enable,
        mem.interrupt_flags
      );
      if mask & 0b00001 != 0 {
        // VBlank
        mem.interrupt_flags &= !0b00001;
        t += cpu.handle_interrupt(mem, 0x40);
      } else if mask & 0b00010 != 0 {
        // LCD Status
        mem.interrupt_flags &= !0b00010;
        t += cpu.handle_interrupt(mem, 0x48);
      } else if mask & 0b00100 != 0 {
        // Timer overflow
        mem.interrupt_flags &= !0b00100;
        t += cpu.handle_interrupt(mem, 0x50);
      } else if mask & 0b01000 != 0 {
        // Serial link
        mem.interrupt_flags &= !0b01000;
        t += cpu.handle_interrupt(mem, 0x58);
      } else if mask & 0b10000 != 0 {
        // Joypad press
        mem.interrupt_flags &= !0b10000;
        t += cpu.handle_interrupt(mem, 0x60);
      }
    }
    if !cpu.ime && mem.interrupt_flags != 0 {
      cpu.halt = false;
    }

    t += cpu.step(mem);
    let ints = mem.step(t);
    mem.interrupt_flags |= ints;

    if ints & 0b00001 != 0 {
      display.redraw(mem.frame());
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
                if let Some(key) = mem::Key::from_code(keycode) {
                  mem.interrupt_flags |= 0b10000;
                  match k.state {
                    glutin::ElementState::Pressed => {
                      mem.key_down(key);
                    }
                    glutin::ElementState::Released => {
                      mem.key_up(key);
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
