#![allow(dead_code)]

use clap::{App, Arg};

#[macro_use]
extern crate log;
extern crate env_logger;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process;

mod cpu;
mod display;
mod gameboy;
mod gpu;
mod mem;

#[derive(Debug)]
struct Args {
  rom: PathBuf,
  test: bool,
}

fn main() {
  match main_result() {
    Ok(_) => (),
    Err(e) => {
      eprintln!("Error: {}", e);
      process::exit(1);
    }
  }
}

fn main_result() -> Result<(), Box<dyn Error>> {
  env_logger::init()?;

  let args = get_args()?;

  let rom = read_file(&args.rom)?;

  let gb = gameboy::GameBoy::new(rom, args.rom)?;
  println!("Starting game: {}", gb.title);
  gb.run(display::Display::new()?, !args.test);
  println!("Thanks for playing!");
  Ok(())
}

fn get_args() -> Result<Args, &'static str> {
  let matches = App::new("GB Rust")
    .version(env!("CARGO_PKG_VERSION"))
    .about("Game Boy emulator")
    .arg(
      Arg::with_name("rom")
        .required(true)
        .help("Path to the Game Boy ROM file to load")
        .value_name("FILE"),
    )
    .arg(
      Arg::with_name("test")
        .required(false)
        .help("Run the emulator in test mode (full speed)")
        .short("t")
        .long("test"),
    )
    .get_matches();

  let rom = PathBuf::from(matches.value_of("rom").unwrap());
  if !rom.is_file() {
    return Err("Provided ROM is a directory");
  }

  Ok(Args {
    rom,
    test: matches.is_present("test"),
  })
}

fn read_file(filename: &Path) -> Result<Vec<u8>, io::Error> {
  let mut file = File::open(filename)?;
  let mut result: Vec<u8> = vec![];
  file.read_to_end(&mut result)?;
  Ok(result)
}
