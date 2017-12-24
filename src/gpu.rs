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
  pub pixels: [[RGBAColor; WIDTH]; HEIGHT],
}

impl GPU {
  pub fn new() -> GPU {
    GPU { pixels: [[COLORS[0]; WIDTH]; HEIGHT] }
  }

  pub fn to_vec(&self) -> Vec<f32> {
    let mut result = vec![];
    for row in self.pixels.iter() {
      for color in row.iter() {
        result.extend(color);
      }
    }
    result
  }

  pub fn step(&mut self) {
    let mut rng = rand::thread_rng();
    for row in self.pixels.iter_mut() {
      for color in row.iter_mut() {
        *color = [rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>(), 1.0];
      }
    }
  }
}
