#![allow(dead_code)]

#![cfg_attr(test, allow(dead_code))]

extern crate glium;
use self::glium::glutin;

use std::{thread, time};

mod cpu;
mod gpu;
mod display;

fn main() {
  let mut display = display::Display::new();
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
    display.redraw();
    display.gpu.step();

    thread::sleep(time::Duration::from_millis(100));
  }
}
