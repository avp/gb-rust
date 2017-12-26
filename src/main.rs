#![allow(dead_code)]
#![cfg_attr(test, allow(dead_code))]

extern crate clap;
use clap::{App, Arg};

extern crate glium;
use glium::glutin;

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

  let cpu = cpu::CPU::new();
  let mem = mem::Memory::new(rom);
  let display = display::Display::new();

  run(cpu, mem, display);
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

fn run(_: cpu::CPU, mut mem: mem::Memory, mut display: display::Display) {
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
      _ => (),
    });

    let do_render = mem.gpu.step(4);
    if do_render {
      display.redraw(&*mem.gpu.frame);
    }
  }
}
