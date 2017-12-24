extern crate rand;
use self::rand::Rng;

/// RGBA Color.
pub type RGBAColor = [f32; 4];

const COLORS: [RGBAColor; 4] =
  [
    [1.0, 1.0, 1.0, 1.0],
    [192.0 / 255.0, 192.0 / 255.0, 192.0 / 255.0, 1.0],
    [96.0 / 255.0, 96.0 / 255.0, 96.0 / 255.0, 1.0],
    [0.0, 0.0, 0.0, 0.0],
  ];

pub const HEIGHT: usize = 144;
pub const WIDTH: usize = 160;

pub struct GPU {
  pub pixels: [f32; 4 * WIDTH * HEIGHT],
}

impl GPU {
  pub fn new() -> GPU {
    GPU { pixels: [1.0; 4 * WIDTH * HEIGHT] }
  }

  pub fn step(&mut self) {
    let mut rng = rand::thread_rng();
    for value in self.pixels.iter_mut() {
      *value = rng.gen::<f32>();
    }
  }
}
