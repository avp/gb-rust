/// RGBA Color.
pub type RGBAColor = (u8, u8, u8, u8);

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
  bgtile: bool,
  switchbg: bool,
  switchlcd: bool,
  scx: u8,
  scy: u8,
  palette: [u8; 4],

  tileset: Box<[Tile; NUM_TILES]>,
}

impl GPU {
  pub fn new() -> GPU {
    GPU {
      frame: Box::new([0; 4 * WIDTH * HEIGHT]),
      render: Box::new([0; WIDTH * HEIGHT]),

      vram: vec![0; VRAM_SIZE],
      oam: vec![0; OAM_SIZE],

      mode: Mode::HBlank,
      mode_clock: 0,
      line: 0,

      bgmap: false,
      bgtile: false,
      switchbg: false,
      switchlcd: false,
      scx: 0,
      scy: 0,
      palette: [255, 192, 96, 0],

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
          // println!("LINE = {}", self.line);
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
          // println!("LINE = {}", self.line);
          // VBlank takes 10 lines to run.
          if self.line > (HEIGHT - 1) + 10 {
            self.mode = Mode::OAMRead;
            self.line = 0;
            // println!("LINE = {}", self.line);
          }
        }
      }
    }
    false
  }

  pub fn update_tile(&mut self, addr: u16) {
    // Base address for this tile row.
    let addr = (addr & 0x1ffe) as usize;

    let tile = ((addr >> 4) & 0x1ff) as usize;
    let row = ((addr >> 1) & 0x7) as usize;

    if tile >= NUM_TILES {
      return;
    }

    debug!("Updating for address: 0x{:x}", addr);

    for col in 0..8 {
      let sx: u8 = 1 << (7 - col);

      let color = if self.vram[addr] & sx != 0 { 1 } else { 0 } +
        if self.vram[addr + 1] & sx != 0 { 2 } else { 0 };
      self.tileset[tile][row][col] = color;
    }

    println!(
      "UPDATETILE: {} ADDR: 0x{:x} TILE={:?}",
      tile,
      addr,
      self.tileset[tile]
    );
  }

  pub fn rb(&self, addr: u16) -> u8 {
    match addr {
      0xff40 => {
        (if self.switchbg { 1 } else { 0 }) | (if self.bgmap { 8 } else { 0 }) |
          (if self.bgtile { 0x10 } else { 0 }) |
          (if self.switchlcd { 0x80 } else { 0 })
      }
      0xff42 => self.scy,
      0xff43 => self.scx,
      0xff44 => self.line as u8,
      _ => 0,
    }
  }

  pub fn wb(&mut self, addr: u16, value: u8) {
    match addr {
      0xff40 => {
        self.switchbg = (value & 0x1) != 0;
        self.bgmap = (value & 0x8) != 0;
        self.bgtile = (value & 0x10) != 0;
        self.switchlcd = (value & 0x80) != 0;
      }
      0xff42 => self.scy = value,
      0xff43 => self.scx = value,
      0xff47 => {
        for i in 0..4 {
          match (value >> (i * 2)) & 3 {
            0 => self.palette[i] = 255,
            1 => self.palette[i] = 192,
            2 => self.palette[i] = 96,
            3 => self.palette[i] = 0,
            _ => unimplemented!(),
          }
        }
      }
      _ => (),
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
    // Tile coordinate top left corner of the background.
    let row = (self.line + self.scy as usize) % 8;
    let mut col = (self.scx % 8) as usize;

    let mapoffs = if self.bgmap { 0x1c00 } else { 0x1800 } +
      (((self.line + self.scy as usize) % 256) >> 3) * 32;

    let mut lineoffs = (self.scy >> 3) as usize;
    let mut tile = self.vram[mapoffs + lineoffs] as usize;

    if self.bgtile && (tile as i16) < 0 {
      println!("BGTILE BEFORE={}", tile);
      tile = tile + 256;
      println!("BGTILE AFTER={}", tile);
    }

    let line = self.line;
    for i in 0..WIDTH {
      let color = self.tileset[tile][row][col];
      self.set(line, i, color);

      if !self.bgmap && self.line == 0 {
        println!("x = {} TILENR = {}", i, tile);
      }

      col += 1;
      if col == 8 {
        // Read another tile since this one is done.
        col = 0;
        lineoffs += 1;
        tile = self.vram[mapoffs + lineoffs] as usize;
        if self.bgtile && (tile as i16) < 0 {
          tile = tile + 256;
        }
      }
    }
  }

  fn render_frame(&mut self) {
    for i in 0..(WIDTH * HEIGHT) {
      let color = self.render[i] as usize;
      let j = i * 4;
      self.frame[j] = self.palette[color];
      self.frame[j + 1] = self.palette[color];
      self.frame[j + 2] = self.palette[color];
      // Full alpha value.
      self.frame[j + 3] = 255;
    }
  }
}
