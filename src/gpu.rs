/// RGBA Color.
pub type RGBAColor = [u8; 4];

const COLORS: [RGBAColor; 4] = [
  [255, 255, 255, 255],
  [192, 192, 192, 255],
  [96, 96, 96, 255],
  [0, 0, 0, 0],
];

pub const HEIGHT: usize = 144;
pub const WIDTH: usize = 160;

pub const VRAM_SIZE: usize = 0x2000;
pub const OAM_SIZE: usize = 0xa0;

pub type Frame = [u8; 4 * WIDTH * HEIGHT];

#[derive(Debug)]
enum Mode {
  OAMRead = 2,
  VRAMRead = 3,
  HBlank = 0,
  VBlank = 1,
}

pub struct GPU {
  pub frame: Box<Frame>,
  render: Box<Frame>,

  pub vram: Vec<u8>,
  pub oam: Vec<u8>,

  mode: Mode,
  mode_clock: u32,
  line: usize,
}

impl GPU {
  pub fn new() -> GPU {
    GPU {
      frame: Box::new([255; 4 * WIDTH * HEIGHT]),
      render: Box::new([255; 4 * WIDTH * HEIGHT]),

      vram: vec![0; VRAM_SIZE],
      oam: vec![0; OAM_SIZE],

      mode: Mode::HBlank,
      mode_clock: 0,
      line: 0,
    }
  }

  fn lighten(&mut self) {
    for value in self.render.iter_mut() {
      *value = if *value == 255 { 0 } else { *value + 1 };
    }
  }

  /// Step the GPU by t cycles.
  /// Return true if the display must be redrawn.
  pub fn step(&mut self, t: u32) -> bool {
    self.mode_clock += t;

    match self.mode {
      Mode::OAMRead => {
        if self.mode_clock >= 80 {
          self.mode_clock = 0;
          self.mode = Mode::VRAMRead;
        }
      }
      Mode::VRAMRead => {
        if self.mode_clock >= 172 {
          self.mode_clock = 0;
          self.mode = Mode::HBlank;

          self.renderscan();
        }
      }
      Mode::HBlank => {
        if self.mode_clock >= 204 {
          self.mode_clock = 0;
          self.line += 1;
          if self.line == HEIGHT - 1 {
            self.mode = Mode::VBlank;
            self.frame = self.render.clone();
            return true;
          }
        }
      }
      Mode::VBlank => {
        if self.mode_clock >= 456 {
          self.mode_clock = 0;
          self.line += 1;
          // VBlank takes 10 lines to run.
          if self.line > (HEIGHT - 1) + 10 {
            self.mode = Mode::OAMRead;
            self.line = 0;
          }
        }
      }
    }
    false
  }

  fn renderscan(&mut self) {
    self.lighten();
  }
}
