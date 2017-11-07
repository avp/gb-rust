use mem::Memory;

use reg;
use reg::Registers;

#[derive(Debug)]
pub struct CPU {
  pub regs: Registers,
  pub mem: Memory,

  /// Current clock.
  m: u32,
  t: u32,
}


impl CPU {
  pub fn new() -> CPU {
    CPU {
      regs: Registers::new(),
      mem: Memory::new(),
      m: 0,
      t: 0,
    }
  }

  /// Return the next byte at the program counter,
  /// and increment the program counter.
  fn bump(&mut self) -> u8 {
    let result = self.mem.rb(self.regs.pc);
    self.regs.pc += 1;
    result
  }

  /// Execute the next opcode.
  /// Return the m-time taken to run that opcode.
  fn exec(&mut self) -> u32 {
    macro_rules! read_u16_le {
      () => {{
        let a = self.bump();
        let b = self.bump();
        u16::from(a) | (u16::from(b) << 8)
      }}
    }
    macro_rules! ld_nn_n {
      ($reg:ident) => {{
        let imm = self.bump();
        self.regs.$reg = imm;
        2
      }}
    }
    macro_rules! ld_n_nn {
      ($r1:ident, $r2:ident) => {{
        self.regs.$r2 = self.bump();
        self.regs.$r1 = self.bump();
        3
      }}
    }
    macro_rules! ld_r1_r2 {
      ($r1:ident, $r2:ident) => {{
        self.regs.$r1 = self.regs.$r2;
        1
      }}
    }
    macro_rules! ld_r1_r2m {
      ($r1:ident, $r2m:ident) => {{
        self.regs.$r1 = self.mem.rb(self.regs.$r2m());
        2
      }}
    }
    macro_rules! ld_r1m_r2 {
      ($r1m:ident, $r2:ident) => {{
        self.mem.wb(self.regs.$r1m(), self.regs.$r2);
        2
      }}
    }

    macro_rules! push {
      ($r1:ident, $r2:ident) => {{
        self.mem.wb(self.regs.sp - 1, self.regs.$r1);
        self.mem.wb(self.regs.sp - 2, self.regs.$r2);
        self.regs.sp -= 2;
        4
      }}
    }
    macro_rules! pop {
      ($r1:ident, $r2:ident) => {{
        self.regs.$r2 = self.mem.rb(self.regs.sp);
        self.regs.$r1 = self.mem.rb(self.regs.sp+1);
        self.regs.sp += 2;
        3
      }}
    }

    macro_rules! add_a_n {
      ($n:expr) => {{
        let a = self.regs.a;
        let n = $n;
        self.regs.a = a + n;
        self.regs.f = 0;
        self.regs.f |= if a + n == 0 {reg::Z} else {0};
        self.regs.f |= if (a & 0xf) + (n & 0xf) > 0xf {reg::H} else {0};
        self.regs.f |=
          if u16::from(a) + u16::from(n) > 0xff {reg::C} else {0};
        1
      }}
    }
    macro_rules! adc_a_n {
      ($n:expr) => {{
        let a = self.regs.a;
        let n = $n;
        let c = if self.regs.c() {1} else {0};
        self.regs.a = a + n + c;
        self.regs.f = 0;
        self.regs.f |= if a + n + c == 0 {reg::Z} else {0};
        self.regs.f |=
          if (a & 0xf) + (n & 0xf) + c > 0xf {reg::H} else {0};
        self.regs.f |=
          if u16::from(a) + u16::from(n) + u16::from(c) > 0xff {reg::C}
          else {0};
        1
      }}
    }

    macro_rules! sub_a_n {
      ($n:expr) => {{
        let a = self.regs.a;
        let n = $n;
        self.regs.a = a - n;
        self.regs.f = reg::N;
        self.regs.f |= if a - n == 0 {reg::Z} else {0};
        self.regs.f |= if a < n {reg::C} else {0};
        self.regs.f |= if (a & 0xf) < (n & 0xf) {reg::H} else {0};
        1
      }}
    }
    macro_rules! sbc_a_n {
      ($n:expr) => {{
        let a = self.regs.a;
        let n = $n;
        let c = if self.regs.c() {1} else {0};
        self.regs.a = a - (n + c);
        self.regs.f = reg::N;
        self.regs.f |= if a - (n + c) == 0 {reg::Z} else {0};
        self.regs.f |= if a < (n + c) {reg::C} else {0};
        self.regs.f |= if (a & 0xf) < ((n + c) & 0xf) {reg::H} else {0};
        1
      }}
    }

    macro_rules! and_a_n {
      ($n:expr) => {{
        self.regs.a &= $n;
        self.regs.f = if self.regs.a == 0 {reg::Z} else {0} | reg::H;
        1
      }}
    }
    macro_rules! or_a_n {
      ($n:expr) => {{
        self.regs.a |= $n;
        self.regs.f = if self.regs.a == 0 {reg::Z} else {0};
        1
      }}
    }

    macro_rules! swap {
      ($reg:ident) => {{
        let top = self.regs.$reg >> 4;
        let bot = self.regs.$reg & 0xf;
        self.regs.$reg = (bot << 4) | top;
        self.regs.f = 0;
        if self.regs.$reg == 0 {
          self.regs.f |= reg::Z;
        }
        2
      }}
    }

    match self.bump() {
      0x00 => 1, // nop
      0x01 => ld_n_nn!(b, c),
      0x02 => ld_r1m_r2!(bc, a),
      0x03 => unimplemented!(),
      0x04 => unimplemented!(),
      0x05 => unimplemented!(),
      0x06 => ld_nn_n!(b),
      0x07 => unimplemented!(),
      0x08 => {
        let nn = read_u16_le!();
        let val = self.mem.rb(self.regs.sp);
        self.mem.wb(nn, val);
        5
      }
      0x09 => unimplemented!(),
      0x0a => ld_r1_r2m!(a, bc),
      0x0b => unimplemented!(),
      0x0c => unimplemented!(),
      0x0d => unimplemented!(),
      0x0e => ld_nn_n!(c),
      0x0f => unimplemented!(),

      0x10 => unimplemented!(),
      0x11 => ld_n_nn!(d, e),
      0x12 => ld_r1m_r2!(de, a),
      0x13 => unimplemented!(),
      0x14 => unimplemented!(),
      0x15 => unimplemented!(),
      0x16 => ld_nn_n!(d),
      0x17 => unimplemented!(),
      0x18 => unimplemented!(),
      0x19 => unimplemented!(),
      0x1a => ld_r1_r2m!(a, de),
      0x1b => unimplemented!(),
      0x1c => unimplemented!(),
      0x1d => unimplemented!(),
      0x1e => ld_nn_n!(e),
      0x1f => unimplemented!(),

      0x20 => unimplemented!(),
      0x21 => ld_n_nn!(h, l),
      0x22 => {
        ld_r1m_r2!(hl, a);
        self.regs.hl_inc();
        2
      }
      0x23 => unimplemented!(),
      0x24 => unimplemented!(),
      0x25 => unimplemented!(),
      0x26 => ld_nn_n!(h),
      0x27 => unimplemented!(),
      0x28 => unimplemented!(),
      0x29 => unimplemented!(),
      0x2a => {
        ld_r1_r2m!(a, hl);
        self.regs.hl_inc();
        2
      }
      0x2b => unimplemented!(),
      0x2c => unimplemented!(),
      0x2d => unimplemented!(),
      0x2e => ld_nn_n!(l),
      0x2f => unimplemented!(),

      0x30 => unimplemented!(),
      0x31 => {
        self.regs.sp = read_u16_le!();
        3
      }
      0x32 => {
        ld_r1m_r2!(hl, a);
        self.regs.hl_dec();
        2
      }
      0x33 => unimplemented!(),
      0x34 => unimplemented!(),
      0x35 => unimplemented!(),
      0x36 => {
        let n = self.bump();
        self.mem.wb(self.regs.hl(), n);
        3
      }
      0x37 => unimplemented!(),
      0x38 => unimplemented!(),
      0x39 => unimplemented!(),
      0x3a => {
        ld_r1_r2m!(a, hl);
        self.regs.hl_dec();
        2
      }
      0x3b => unimplemented!(),
      0x3c => unimplemented!(),
      0x3d => unimplemented!(),
      0x3e => {
        self.regs.a = self.bump();
        2
      }
      0x3f => unimplemented!(),

      0x40 => ld_r1_r2!(b, b),
      0x41 => ld_r1_r2!(b, c),
      0x42 => ld_r1_r2!(b, d),
      0x43 => ld_r1_r2!(b, e),
      0x44 => ld_r1_r2!(b, h),
      0x45 => ld_r1_r2!(b, l),
      0x46 => ld_r1_r2m!(b, hl),
      0x47 => unimplemented!(),
      0x48 => ld_r1_r2!(c, b),
      0x49 => ld_r1_r2!(c, c),
      0x4a => ld_r1_r2!(c, d),
      0x4b => ld_r1_r2!(c, e),
      0x4c => ld_r1_r2!(c, h),
      0x4d => ld_r1_r2!(c, l),
      0x4e => ld_r1_r2m!(c, hl),
      0x4f => unimplemented!(),

      0x50 => ld_r1_r2!(d, b),
      0x51 => ld_r1_r2!(d, c),
      0x52 => ld_r1_r2!(d, d),
      0x53 => ld_r1_r2!(d, e),
      0x54 => ld_r1_r2!(d, h),
      0x55 => ld_r1_r2!(d, l),
      0x56 => ld_r1_r2m!(d, hl),
      0x57 => unimplemented!(),
      0x58 => ld_r1_r2!(e, b),
      0x59 => ld_r1_r2!(e, c),
      0x5a => ld_r1_r2!(e, d),
      0x5b => ld_r1_r2!(e, e),
      0x5c => ld_r1_r2!(e, h),
      0x5d => ld_r1_r2!(e, l),
      0x5e => ld_r1_r2m!(e, hl),
      0x5f => unimplemented!(),

      0x60 => ld_r1_r2!(h, b),
      0x61 => ld_r1_r2!(h, c),
      0x62 => ld_r1_r2!(h, d),
      0x63 => ld_r1_r2!(h, e),
      0x64 => ld_r1_r2!(h, h),
      0x65 => ld_r1_r2!(h, l),
      0x66 => ld_r1_r2m!(h, hl),
      0x67 => unimplemented!(),
      0x68 => ld_r1_r2!(l, b),
      0x69 => ld_r1_r2!(l, c),
      0x6a => ld_r1_r2!(l, d),
      0x6b => ld_r1_r2!(l, e),
      0x6c => ld_r1_r2!(l, h),
      0x6d => ld_r1_r2!(l, l),
      0x6e => ld_r1_r2m!(l, hl),
      0x6f => unimplemented!(),

      0x70 => ld_r1m_r2!(hl, b),
      0x71 => ld_r1m_r2!(hl, c),
      0x72 => ld_r1m_r2!(hl, d),
      0x73 => ld_r1m_r2!(hl, e),
      0x74 => ld_r1m_r2!(hl, h),
      0x75 => ld_r1m_r2!(hl, l),
      0x76 => unimplemented!(),
      0x77 => ld_r1m_r2!(hl, a),
      0x78 => ld_r1_r2!(a, b),
      0x79 => ld_r1_r2!(a, c),
      0x7a => ld_r1_r2!(a, d),
      0x7b => ld_r1_r2!(a, e),
      0x7c => ld_r1_r2!(a, h),
      0x7d => ld_r1_r2!(a, l),
      0x7e => ld_r1_r2m!(a, hl),
      0x7f => ld_r1_r2!(a, a),

      0x80 => add_a_n!(self.regs.b),
      0x81 => add_a_n!(self.regs.c),
      0x82 => add_a_n!(self.regs.d),
      0x83 => add_a_n!(self.regs.e),
      0x84 => add_a_n!(self.regs.h),
      0x85 => add_a_n!(self.regs.l),
      0x86 => add_a_n!(self.mem.rb(self.regs.hl())),
      0x87 => {
        add_a_n!(self.regs.a);
        2
      }
      0x88 => adc_a_n!(self.regs.b),
      0x89 => adc_a_n!(self.regs.c),
      0x8a => adc_a_n!(self.regs.d),
      0x8b => adc_a_n!(self.regs.e),
      0x8c => adc_a_n!(self.regs.h),
      0x8d => adc_a_n!(self.regs.l),
      0x8e => {
        adc_a_n!(self.mem.rb(self.regs.hl()));
        2
      }
      0x8f => adc_a_n!(self.regs.a),

      0x90 => sub_a_n!(self.regs.b),
      0x91 => sub_a_n!(self.regs.c),
      0x92 => sub_a_n!(self.regs.d),
      0x93 => sub_a_n!(self.regs.e),
      0x94 => sub_a_n!(self.regs.h),
      0x95 => sub_a_n!(self.regs.l),
      0x96 => {
        sub_a_n!(self.mem.rb(self.regs.hl()));
        2
      }
      0x97 => sbc_a_n!(self.regs.a),
      0x98 => sbc_a_n!(self.regs.b),
      0x99 => sbc_a_n!(self.regs.c),
      0x9a => sbc_a_n!(self.regs.d),
      0x9b => sbc_a_n!(self.regs.e),
      0x9c => sbc_a_n!(self.regs.h),
      0x9d => sbc_a_n!(self.regs.l),
      0x9e => {
        sbc_a_n!(self.mem.rb(self.regs.hl()));
        2
      }
      0x9f => sbc_a_n!(self.regs.a),

      0xa0 => and_a_n!(self.regs.b),
      0xa1 => and_a_n!(self.regs.c),
      0xa2 => and_a_n!(self.regs.d),
      0xa3 => and_a_n!(self.regs.e),
      0xa4 => and_a_n!(self.regs.h),
      0xa5 => and_a_n!(self.regs.l),
      0xa6 => {
        and_a_n!(self.mem.rb(self.regs.hl()));
        2
      }
      0xa7 => and_a_n!(self.regs.a),
      0xa8 => unimplemented!(),
      0xa9 => unimplemented!(),
      0xaa => unimplemented!(),
      0xab => unimplemented!(),
      0xac => unimplemented!(),
      0xad => unimplemented!(),
      0xae => unimplemented!(),
      0xaf => unimplemented!(),

      0xb0 => or_a_n!(self.regs.b),
      0xb1 => or_a_n!(self.regs.c),
      0xb2 => or_a_n!(self.regs.d),
      0xb3 => or_a_n!(self.regs.e),
      0xb4 => or_a_n!(self.regs.h),
      0xb5 => or_a_n!(self.regs.l),
      0xb6 => {
        or_a_n!(self.mem.rb(self.regs.hl()));
        2
      }
      0xb7 => or_a_n!(self.regs.a),
      0xb8 => unimplemented!(),
      0xb9 => unimplemented!(),
      0xba => unimplemented!(),
      0xbb => unimplemented!(),
      0xbc => unimplemented!(),
      0xbd => unimplemented!(),
      0xbe => unimplemented!(),
      0xbf => unimplemented!(),

      0xc0 => unimplemented!(),
      0xc1 => pop!(b, c),
      0xc2 => unimplemented!(),
      0xc3 => unimplemented!(),
      0xc4 => unimplemented!(),
      0xc5 => push!(b, c),
      0xc6 => {
        add_a_n!(self.bump());
        2
      }
      0xc7 => unimplemented!(),
      0xc8 => unimplemented!(),
      0xc9 => unimplemented!(),
      0xca => unimplemented!(),
      0xcb => {
        match self.bump() {
          0x30 => swap!(b),
          0x31 => swap!(c),
          0x32 => swap!(d),
          0x33 => swap!(e),
          0x34 => swap!(h),
          0x35 => swap!(l),
          0x36 => {
            let byte = self.mem.rb(self.regs.hl());
            let top = byte >> 4;
            let bot = byte & 0xf;
            self.mem.wb(self.regs.hl(), (bot << 4) | top);
            4
          }
          0x37 => swap!(a),
          _ => panic!("Unsupported opcode in swap"),
        }
      }
      0xcc => unimplemented!(),
      0xcd => unimplemented!(),
      0xce => {
        adc_a_n!(self.bump());
        2
      }
      0xcf => unimplemented!(),

      0xd0 => unimplemented!(),
      0xd1 => pop!(d, e),
      0xd2 => unimplemented!(),
      0xd3 => unimplemented!(),
      0xd4 => unimplemented!(),
      0xd5 => push!(d, e),
      0xd6 => {
        sub_a_n!(self.bump());
        2
      }
      0xd7 => unimplemented!(),
      0xd8 => unimplemented!(),
      0xd9 => unimplemented!(),
      0xda => unimplemented!(),
      0xdb => unimplemented!(),
      0xdc => unimplemented!(),
      0xdd => unimplemented!(),
      0xde => unimplemented!(),
      0xdf => unimplemented!(),

      0xe0 => {
        let n = self.bump();
        self.mem.wb(0xff00 + u16::from(n), self.regs.a);
        3
      }
      0xe1 => pop!(h, l),
      0xe2 => {
        self.mem.wb(0xff00 + u16::from(self.regs.c), self.regs.a);
        2
      }
      0xe3 => unimplemented!(),
      0xe4 => unimplemented!(),
      0xe5 => push!(h, l),
      0xe6 => {
        and_a_n!(self.bump());
        2
      }
      0xe7 => unimplemented!(),
      0xe8 => unimplemented!(),
      0xe9 => unimplemented!(),
      0xea => {
        let nn = read_u16_le!();
        self.mem.wb(nn, self.regs.a);
        4
      }
      0xeb => unimplemented!(),
      0xec => unimplemented!(),
      0xed => unimplemented!(),
      0xee => unimplemented!(),
      0xef => unimplemented!(),

      0xf0 => {
        let n = self.bump();
        self.regs.a = self.mem.rb(0xff00 + u16::from(n));
        3
      }
      0xf1 => pop!(a, f),
      0xf2 => {
        self.regs.a = self.mem.rb(0xff00 + u16::from(self.regs.c));
        2
      }
      0xf3 => unimplemented!(),
      0xf4 => unimplemented!(),
      0xf5 => push!(a, f),
      0xf6 => {
        or_a_n!(self.bump());
        2
      }
      0xf7 => unimplemented!(),
      0xf8 => unimplemented!(),
      0xf9 => {
        self.regs.sp = self.regs.hl();
        2
      }
      0xfa => {
        let nn = read_u16_le!();
        self.regs.a = self.mem.rb(nn);
        4
      }
      0xfb => unimplemented!(),
      0xfc => unimplemented!(),
      0xfd => unimplemented!(),
      0xfe => unimplemented!(),
      0xff => unimplemented!(),
      _ => panic!("Invalid opcode"),
    }
  }
}

#[cfg(test)]
mod tests {}
