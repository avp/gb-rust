#![allow(dead_code)]

#![cfg_attr(test, allow(dead_code))]

extern crate glium;
use self::glium::glutin;

mod cpu;
mod mem;
mod gpu;
mod display;

fn main() {
  let cpu = cpu::CPU::new();
  let mem = mem::Memory::new();
  let display = display::Display::new();

  run(cpu, mem, display);
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
