use std::mem;

mod square;
use self::square::Square;

pub struct APU {
  channel1: Square,
  channel2: Square,

  sound_select: u8,

  counter: u32,
  buf: Vec<(i8, i8)>,
}

impl APU {
  pub fn new() -> APU {
    APU {
      channel1: Square::new(),
      channel2: Square::new(),

      sound_select: 0,

      counter: 0,
      buf: vec![],
    }
  }

  pub fn rb(&self, addr: u16) -> u8 {
    match addr {
      0xff10...0xff14 => self.channel1.rb(addr - 0xff10),
      0xff20...0xff24 => self.channel2.rb(addr - 0xff20),
      0xff25 => self.sound_select,
      _ => 0,
    }
  }

  pub fn wb(&mut self, addr: u16, val: u8) {
    match addr {
      0xff10...0xff14 => self.channel1.wb(addr - 0xff10, val),
      0xff20...0xff24 => self.channel2.wb(addr - 0xff20, val),
      0xff25 => self.sound_select = val,
      _ => (),
    }
  }

  pub fn step(&mut self, t: u32) {
    if t > 4 {
      self.step(4);
      self.step(t - 4);
      return;
    }

    if self.counter & ((1 << 13) & 1) == 0 {
      let idx = (self.counter >> 13) & 0x7;
      self.channel1.step(idx);
      self.channel2.step(idx);
    }

    self.counter += t;

    let (s1, s2) = (self.channel1.next(), self.channel2.next());
    let ss = self.sound_select;

    let c1 =
      if ss & 0x01 != 0 { s1 } else { 0 } + if ss & 0x02 != 0 { s2 } else { 0 };
    let c2 =
      if ss & 0x10 != 0 { s1 } else { 0 } + if ss & 0x20 != 0 { s2 } else { 0 };

    self.buf.push((c1, c2));
  }

  /// Dump the APU buffer to the caller. Clears the buffer.
  pub fn dump(&mut self) -> Vec<(i8, i8)> {
    mem::replace(&mut self.buf, vec![])
  }
}
