#![allow(dead_code)]
#![cfg_attr(test, allow(dead_code))]

extern crate clap;
use clap::{App, Arg};

extern crate glium;
use glium::glutin;

#[macro_use]
extern crate log;
extern crate env_logger;

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
  env_logger::init().unwrap();

  let args = get_args();

  let rom;
  println!("Loading ROM: {}", &args.rom);
  match read_file(&args.rom) {
    Ok(r) => {
      rom = r;
      println!("ROM Loaded: {}", &args.rom);
    }
    Err(e) => {
      eprintln!("{}", e);
      process::exit(1);
    }
  }

  let mut cpu = cpu::CPU::new();
  let mut mem = mem::Memory::new(rom);
  let mut display = display::Display::new();

  run(&mut cpu, &mut mem, &mut display);
}

fn get_args() -> Args {
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

  Args { rom: String::from(matches.value_of("rom").unwrap()) }
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

    let t = cpu.step(mem);
    let do_render = mem.gpu.step(t);
    if do_render {
      debug!("Rendering frame");
      display.redraw(&*mem.gpu.frame);
    }
  }
}
