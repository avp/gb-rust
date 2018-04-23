/// RGBA Color.
pub type RGBAColor = (u8, u8, u8, u8);

pub const HEIGHT: usize = 144;
pub const WIDTH: usize = 160;

const TILEMAP_HEIGHT: usize = 32;
const TILEMAP_WIDTH: usize = 32;

pub const VRAM_SIZE: usize = 0x2000;
pub const OAM_SIZE: usize = 0xa0;

const NUM_TILES: usize = 384;
const NUM_OBJECTS: usize = 40;

pub type Frame = [u8; 4 * WIDTH * HEIGHT];

const COLORS: [u8; 4] = [255, 150, 50, 0];

#[derive(Debug, Copy, Clone)]
enum Mode {
  OAMRead = 2,
  VRAMRead = 3,
  HBlank = 0,
  VBlank = 1,
}

type Tile = [[u8; 8]; 8];

#[derive(Debug, Copy, Clone)]
struct Object {
  pub x: i32,
  pub y: i32,
  pub tile: usize,
  pub palette: bool,
  pub xflip: bool,
  pub yflip: bool,
  pub priority: bool,
  pub num: u32,
}

impl Object {
  fn new() -> Object {
    Object {
      x: -8,
      y: -16,
      tile: 0,
      palette: false,
      xflip: false,
      yflip: false,
      priority: false,
      num: 0,
    }
  }
}

pub struct GPU {
  pub frame: Box<Frame>,
  pub render: Box<[u8; WIDTH * HEIGHT]>,

  pub vram: Vec<u8>,
  pub oam: Vec<u8>,

  mode: Mode,
  mode_clock: u32,
  line: usize,
  lyc: u8,

  bgmap: bool,
  bgtile: bool,
  switchbg: bool,
  switchlcd: bool,
  scx: u8,
  scy: u8,
  bg_palette: [u8; 4],

  switchobj: bool,
  objsize: bool,
  obj0_palette: [u8; 4],
  obj1_palette: [u8; 4],

  switchwin: bool,
  winmap: bool,
  winx: u8,
  winy: u8,

  lycly: bool,
  mode0int: bool,
  mode1int: bool,
  mode2int: bool,

  tileset: Box<[Tile; NUM_TILES]>,
  objects: Box<[Object; NUM_OBJECTS]>,
}

impl GPU {
  pub fn new() -> GPU {
    let mut objects = [Object::new(); NUM_OBJECTS];
    for i in 0..NUM_OBJECTS {
      objects[i].num = i as u32;
    }

    GPU {
      frame: Box::new([0; 4 * WIDTH * HEIGHT]),
      render: Box::new([0; WIDTH * HEIGHT]),

      vram: vec![0; VRAM_SIZE],
      oam: vec![0; OAM_SIZE],

      mode: Mode::HBlank,
      mode_clock: 0,
      line: 0,
      lyc: 0,

      bgmap: false,
      bgtile: false,
      switchbg: false,
      switchlcd: false,
      scx: 0,
      scy: 0,
      bg_palette: COLORS.clone(),

      switchobj: false,
      objsize: false,
      obj0_palette: COLORS.clone(),
      obj1_palette: COLORS.clone(),

      switchwin: false,
      winmap: false,
      winx: 0,
      winy: 0,

      lycly: false,
      mode0int: false,
      mode1int: false,
      mode2int: false,

      tileset: Box::new([[[0; 8]; 8]; NUM_TILES]),
      objects: Box::new(objects),
    }
  }

  /// Step the GPU by t cycles.
  /// Return interrupt flags that have been set.
  pub fn step(&mut self, t: u32) -> u8 {
    if !self.switchlcd {
      return 0;
    }

    self.mode_clock += t;

    let mut int = 0;

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
          if self.mode0int {
            int |= 0x02;
          }
        }
      }
      Mode::HBlank => {
        if self.mode_clock >= 204 {
          self.mode_clock = 0;
          self.line += 1;
          if self.lycly && self.lyc as usize == self.line {
            int |= 0x02;
          }
          if self.line == HEIGHT - 1 {
            self.mode = Mode::VBlank;
            self.render_frame();
            int |= 0x01;
            if self.mode1int {
              int |= 0x02;
            }
          } else {
            self.mode = Mode::OAMRead;
            if self.mode2int {
              int |= 0x02;
            }
          }
        }
      }
      Mode::VBlank => {
        if self.mode_clock >= 456 {
          self.mode_clock = 0;
          self.line += 1;
          if self.lycly && self.lyc as usize == self.line {
            int |= 0x02;
          }
          // VBlank takes 10 lines to run.
          if self.line > (HEIGHT - 1) + 10 {
            self.mode = Mode::OAMRead;
            self.line = 0;
            if self.mode2int {
              int |= 0x02;
            }
          }
        }
      }
    }
    int
  }

  pub fn update_tile(&mut self, addr: u16) {
    // Base address for this tile row.
    let addr = (addr & 0x1ffe) as usize;

    let tile = (addr / 16) as usize;
    let row = ((addr / 2) % 8) as usize;

    if tile >= NUM_TILES {
      return;
    }

    for col in 0..8 {
      let sx: u8 = 1 << (7 - col);

      let color = if self.vram[addr] & sx != 0 {
        1
      } else {
        0
      } + if self.vram[addr + 1] & sx != 0 {
        2
      } else {
        0
      };
      self.tileset[tile][row][col] = color;
    }
  }

  pub fn update_object(&mut self, addr: u16, val: u8) {
    let i = ((addr & 0xff) / 4) as usize;
    if i < NUM_OBJECTS {
      match addr % 4 {
        0 => self.objects[i].y = (val as u16 as i32) - 16,
        1 => self.objects[i].x = (val as u16 as i32) - 8,
        2 => self.objects[i].tile = val as usize,
        3 => {
          self.objects[i].palette = val & 0x10 != 0;
          self.objects[i].xflip = val & 0x20 != 0;
          self.objects[i].yflip = val & 0x40 != 0;
          self.objects[i].priority = val & 0x80 == 0;
        }
        _ => panic!("addr % 4 > 3"),
      }
      debug!("Updated object {}: {:?}", i, self.objects[i]);
    }
  }

  pub fn rb(&self, addr: u16) -> u8 {
    match addr {
      0xff40 => {
        (if self.switchbg { 0x01 } else { 0 }) | (if self.switchobj {
          0x02
        } else {
          0
        }) | (if self.objsize { 0x04 } else { 0 }) | (if self.bgmap {
          0x08
        } else {
          0
        }) | (if self.bgtile { 0x10 } else { 0 }) | (if self.switchwin {
          0x20
        } else {
          0
        }) | (if self.winmap { 0x40 } else { 0 }) | (if self.switchlcd {
          0x80
        } else {
          0
        })
      }
      0xff41 => {
        ((self.lycly as u8) << 6) | ((self.mode2int as u8) << 5)
          | ((self.mode1int as u8) << 4) | ((self.mode0int as u8) << 3)
          | (if self.lyc as usize == self.line {
            1 << 2
          } else {
            0
          }) | (self.mode as u8)
      }
      0xff42 => self.scy,
      0xff43 => self.scx,
      0xff44 => self.line as u8,
      0xff45 => self.lyc,
      0xff4a => self.winy,
      0xff4b => self.winx,
      _ => 0,
    }
  }

  pub fn wb(&mut self, addr: u16, value: u8) {
    match addr {
      0xff40 => {
        self.switchbg = (value & 0x01) != 0;
        self.switchobj = (value & 0x02) != 0;
        self.objsize = (value & 0x04) != 0;
        self.bgmap = (value & 0x08) != 0;
        self.bgtile = (value & 0x10) != 0;
        self.switchwin = (value & 0x20) != 0;
        self.winmap = (value & 0x40) != 0;
        self.switchlcd = (value & 0x80) != 0;
      }
      0xff41 => {
        self.lycly = (value >> 6) & 1 != 0;
        self.mode2int = (value >> 5) & 1 != 0;
        self.mode1int = (value >> 4) & 1 != 0;
        self.mode0int = (value >> 3) & 1 != 0;
      }
      0xff42 => self.scy = value,
      0xff43 => self.scx = value,
      0xff45 => self.lyc = value,
      0xff46 => panic!("0xff46 write to GPU is invalid"),
      0xff47...0xff49 => {
        let pal = match addr {
          0xff47 => &mut self.bg_palette,
          0xff48 => &mut self.obj0_palette,
          0xff49 => &mut self.obj1_palette,
          _ => panic!(),
        };
        for i in 0..4 {
          match (value >> (i * 2)) & 3 {
            0 => pal[i] = COLORS[0],
            1 => pal[i] = COLORS[1],
            2 => pal[i] = COLORS[2],
            3 => pal[i] = COLORS[3],
            _ => unimplemented!(),
          }
        }
      }
      0xff4a => self.winy = value,
      0xff4b => self.winx = value,
      _ => (),
    }
  }

  fn render_line(&mut self) {
    let mut scanrow = [0u8; WIDTH];

    if self.switchbg {
      self.render_bg(&mut scanrow);
    }

    if self.switchwin {
      self.render_window(&mut scanrow);
    }

    if self.switchobj {
      self.render_objects(&mut scanrow);
    }
  }

  fn render_bg(&mut self, scanrow: &mut [u8; WIDTH]) {
    // Tile coordinate top left corner of the background.
    let row = (self.line + self.scy as usize) % 8;
    let mut col = (self.scx % 8) as usize;

    let map_base = if self.bgmap { 0x1c00 } else { 0x1800 };

    // Current screen line number + vertical scroll offset
    // is the line of the bg.
    // Confine it to the 256 possible tiles to ensure wraparound,
    // divide by 8 pixels per tile,
    // and multiply by TILEMAP_WIDTH tiles in each previous row of the map.
    let map_row_offset = map_base
      + ((((self.line + self.scy as usize) % 256) / 8) * TILEMAP_WIDTH);

    // Add to that the horizontal offset (just offset / 8 pixels per tile).
    let mut map_col_offset = ((self.scx / 8) as usize % TILEMAP_WIDTH) as usize;
    let mut tile = self.vram[map_row_offset + map_col_offset] as u16;
    if !self.bgtile {
      tile = (tile as i8 as i16 + 256) as u16;
    };

    let line = self.line;
    for i in 0..WIDTH {
      let color = self.tileset[tile as usize][row][col];

      self.render[line * WIDTH + i] = self.bg_palette[color as usize];
      scanrow[i] = color;

      col += 1;
      if col == 8 {
        // Read another tile since this one is done.
        col = 0;

        map_col_offset = (map_col_offset + 1) % TILEMAP_WIDTH;
        tile = self.vram[map_row_offset + map_col_offset] as u16;
        if !self.bgtile {
          tile = (tile as i8 as i16 + 256) as u16;
        };
      }
    }
  }

  fn render_window(&mut self, scanrow: &mut [u8; WIDTH]) {
    if self.line < self.winy as usize {
      return;
    }
    let winy = self.line - self.winy as usize;

    if self.winx >= WIDTH as u8 + 7 {
      return;
    }
    let winx = self.winx - 8;

    // Tile coordinate top left corner of the background.
    let row = (self.line - self.winy as usize) % 8;
    let mut col = (winx % 8) as usize;

    let map_base = if self.winmap {
      0x1c00
    } else {
      0x1800
    };

    // Current screen line number + vertical scroll offset
    // is the line of the bg.
    // Confine it to the 256 possible tiles to ensure wraparound,
    // divide by 8 pixels per tile,
    // and multiply by TILEMAP_WIDTH tiles in each previous row of the map.
    let map_row_offset = map_base + (((winy % 256) / 8) * TILEMAP_WIDTH);

    // Add to that the horizontal offset (just offset / 8 pixels per tile).
    let mut map_col_offset = ((winx / 8) as usize % TILEMAP_WIDTH) as usize;
    let mut tile = self.vram[map_row_offset + map_col_offset] as u16;
    if !self.bgtile {
      tile = (tile as i8 as i16 + 256) as u16;
    };

    let line = self.line;
    for i in 0..WIDTH {
      let color = self.tileset[tile as usize][row][col];

      self.render[line * WIDTH + i] = self.bg_palette[color as usize];
      scanrow[i] = color;

      col += 1;
      if col == 8 {
        // Read another tile since this one is done.
        col = 0;

        map_col_offset = (map_col_offset + 1) % TILEMAP_WIDTH;
        tile = self.vram[map_row_offset + map_col_offset] as u16;
        if !self.bgtile {
          tile = (tile as i8 as i16 + 256) as u16;
        };
      }
    }
  }

  fn render_objects(&mut self, scanrow: &mut [u8; WIDTH]) {
    for i in 0..NUM_OBJECTS {
      let line = self.line as i32;
      let ysize = if self.objsize { 16 } else { 8 };

      let object = self.objects[i];
      if object.y <= line && line < (object.y + ysize) {
        debug!(
          "Rendering object {} at line={} {:?}",
          i, self.line, object
        );
        let pal = if object.palette {
          self.obj1_palette
        } else {
          self.obj0_palette
        };

        let (tile, objy) = if ysize == 16 {
          if object.yflip {
            if line - object.y >= 8 {
              (object.tile, object.y + 8)
            } else {
              (object.tile + 1, object.y)
            }
          } else {
            if line - object.y >= 8 {
              (object.tile + 1, object.y + 8)
            } else {
              (object.tile, object.y)
            }
          }
        } else {
          (object.tile, object.y)
        };

        let tilerow = if object.yflip {
          self.tileset[tile][7 - (line - objy) as usize]
        } else {
          self.tileset[tile][(line - objy) as usize]
        };

        for x in 0..8 {
          let screen_idx = object.x + x;
          if 0 <= screen_idx && screen_idx < WIDTH as i32 {
            let tilerow_idx = if object.xflip { 7 - x } else { x } as usize;
            let pal_idx = tilerow[tilerow_idx] as usize;
            if pal_idx != 0 {
              if object.priority || scanrow[screen_idx as usize] == 0 {
                let color = pal[pal_idx];
                let row = self.line;
                let col = screen_idx as usize;
                self.set_color(row, col, color);
              }
            }
          }
        }
      }
    }
  }

  fn render_frame(&mut self) {
    for i in 0..(WIDTH * HEIGHT) {
      let color = self.render[i];
      let j = i * 4;
      self.frame[j] = color;
      self.frame[j + 1] = color;
      self.frame[j + 2] = color;
      // Full alpha value.
      self.frame[j + 3] = 255;
    }
  }

  fn set_color(&mut self, row: usize, col: usize, value: u8) {
    self.render[(row * WIDTH) + col] = value;
  }
}
