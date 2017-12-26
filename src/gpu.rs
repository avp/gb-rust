/// RGBA Color.
pub type RGBAColor = (u8, u8, u8, u8);

/// Map of GB color codes to grayscale values.
const COLORS: [u8; 4] = [255, 192, 96, 0];

pub const HEIGHT: usize = 144;
pub const WIDTH: usize = 160;

pub const VRAM_SIZE: usize = 0x2000;
pub const OAM_SIZE: usize = 0xa0;

const NUM_TILES: usize = 384;

pub type Frame = [u8; 4 * WIDTH * HEIGHT];

#[derive(Debug)]
enum Mode {
  OAMRead = 2,
  VRAMRead = 3,
  HBlank = 0,
  VBlank = 1,
}

type Tile = [[u8; 8]; 8];

pub struct GPU {
  pub frame: Box<Frame>,
  render: Box<[u8; WIDTH * HEIGHT]>,

  pub vram: Vec<u8>,
  pub oam: Vec<u8>,

  mode: Mode,
  mode_clock: u32,
  line: usize,

  bgmap: bool,
  bgtile: usize,
  scx: u8,
  scy: u8,

  tileset: Box<[Tile; NUM_TILES]>,
}

impl GPU {
  pub fn new() -> GPU {
    GPU {
      frame: Box::new([255; 4 * WIDTH * HEIGHT]),
      render: Box::new([0; WIDTH * HEIGHT]),

      vram: vec![0; VRAM_SIZE],
      oam: vec![0; OAM_SIZE],

      mode: Mode::HBlank,
      mode_clock: 0,
      line: 0,

      bgmap: false,
      bgtile: 0,
      scx: 0,
      scy: 0,

      tileset: Box::new([[[0; 8]; 8]; NUM_TILES]),
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

          self.render_line();
        }
      }
      Mode::HBlank => {
        if self.mode_clock >= 204 {
          self.mode_clock = 0;
          self.line += 1;
          if self.line == HEIGHT - 1 {
            self.mode = Mode::VBlank;
            self.render_frame();
            return true;
          } else {
            self.mode = Mode::OAMRead;
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

  pub fn update_tile(&mut self, addr: u16) {
    // Base address for this tile.
    let addr = (addr & 0x1ffe) as usize;

    let tile = ((addr >> 4) & 511) as usize;
    let row = ((addr >> 1) & 7) as usize;

    for col in 0..9 {
      let sx = 1 << (7 - col);

      self.tileset[tile][row][col] =
        if self.vram[addr] & sx != 0 { 1 } else { 0 } +
          if self.vram[addr + 1] & sx != 0 { 2 } else { 0 }
    }
  }

  /// Get the value of the pixel at (row, col).
  fn get(&self, row: usize, col: usize) -> u8 {
    self.render[row * WIDTH + col]
  }

  /// Set the value of the pixel at (row, col).
  fn set(&mut self, row: usize, col: usize, value: u8) {
    assert!(value <= 3);
    self.render[row * WIDTH + col] = value;
  }

  fn render_line(&mut self) {
    let mut mapoffs = if self.bgmap { 0x1c00 } else { 0x1800 };
    mapoffs += ((self.line + self.scy as usize) & 255) >> 3;

    let row = (self.line + self.scy as usize) & 7;
    let mut canvasoffs = self.line * WIDTH;

    let mut col = (self.scx & 7) as usize;
    let mut lineoffs = (self.scx >> 3) as usize;
    let mut tile = self.vram[mapoffs + lineoffs] as usize;
    if self.bgtile == 1 && tile < 128 {
      tile += 256;
    }

    for _ in 0..WIDTH + 1 {
      let color = self.tileset[tile][row][col];
      self.render[canvasoffs] = color;
      canvasoffs += 1;

      col += 1;
      if col == 8 {
        // Read another tile since this one is done.
        col = 0;
        lineoffs = (lineoffs + 1) & 31;
        tile = self.vram[mapoffs + lineoffs] as usize;
        if self.bgtile == 1 && tile < 128 {
          tile += 256;
        }
      }
    }
  }

  fn render_frame(&mut self) {
    for i in 0..(WIDTH * HEIGHT) {
      let color = self.render[i] as usize;
      let j = i * 4;
      self.frame[j] = COLORS[color];
      self.frame[j + 1] = COLORS[color];
      self.frame[j + 2] = COLORS[color];
      // Full alpha value.
      self.frame[j + 3] = 255;
    }
  }
}
