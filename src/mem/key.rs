#[derive(Debug)]
pub enum Key {
  A,
  B,
  Start,
  Select,
  Left,
  Up,
  Down,
  Right,
}

impl Key {
  pub fn from_code(code: minifb::Key) -> Option<Key> {
    match code {
      minifb::Key::Z => Some(Key::A),
      minifb::Key::X => Some(Key::B),
      minifb::Key::Enter => Some(Key::Start),
      minifb::Key::Space => Some(Key::Select),
      minifb::Key::Left => Some(Key::Left),
      minifb::Key::Right => Some(Key::Right),
      minifb::Key::Up => Some(Key::Up),
      minifb::Key::Down => Some(Key::Down),
      _ => None,
    }
  }
}

#[derive(Debug)]
pub struct KeyData {
  rows: (u8, u8),
  column: u8,
}

impl KeyData {
  pub fn new() -> KeyData {
    KeyData {
      rows: (0x0f, 0x0f),
      column: 0,
    }
  }

  pub fn rb(&self) -> u8 {
    match self.column {
      0x10 => self.rows.0,
      0x20 => self.rows.1,
      _ => 0,
    }
  }

  pub fn wb(&mut self, val: u8) {
    self.column = val;
  }

  pub fn key_down(&mut self, key: Key) {
    debug!("Pressed {:?}. Key = {:?}", key, &self);
    match key {
      Key::Right => self.rows.1 &= 0xe,
      Key::Left => self.rows.1 &= 0xd,
      Key::Up => self.rows.1 &= 0xb,
      Key::Down => self.rows.1 &= 0x7,
      Key::A => self.rows.0 &= 0xe,
      Key::B => self.rows.0 &= 0xd,
      Key::Select => self.rows.0 &= 0xb,
      Key::Start => self.rows.0 &= 0x7,
    }
  }

  pub fn key_up(&mut self, key: Key) {
    debug!("Released {:?}. Key = {:?}", key, &self);
    match key {
      Key::Right => self.rows.1 |= 0x1,
      Key::Left => self.rows.1 |= 0x2,
      Key::Up => self.rows.1 |= 0x4,
      Key::Down => self.rows.1 |= 0x8,
      Key::A => self.rows.0 |= 0x1,
      Key::B => self.rows.0 |= 0x2,
      Key::Select => self.rows.0 |= 0x4,
      Key::Start => self.rows.0 |= 0x8,
    }
  }
}
