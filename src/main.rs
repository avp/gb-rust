#![allow(dead_code)]

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
mod gameboy;
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
  env_logger::init()?;

  let args = get_args();

  info!("Reading ROM: {}", &args.rom);
  let rom = read_file(&args.rom)?;

  let mut gb = gameboy::GameBoy::new(rom)?;
  println!("Starting game: {}", gb.title());
  gb.run(&mut display::Display::new())?;
  println!("Thanks for playing!");
  Ok(())
}

fn get_args() -> Args {
  let matches = App::new("GB Rust")
    .version("0.1.0")
    .about("Game Boy emulator")
    .arg(
      Arg::with_name("rom")
        .required(true)
        .help("Path to the Game Boy ROM file to load")
        .value_name("FILE"),
    )
    .get_matches();

  let rom = matches.value_of("rom").unwrap();
  Args { rom: String::from(rom) }
}

fn read_file(filename: &str) -> Result<Vec<u8>, io::Error> {
  let mut file = File::open(filename)?;
  let mut result: Vec<u8> = vec![];
  file.read_to_end(&mut result)?;
  Ok(result)
}
