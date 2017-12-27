use cpu::CPU;
use cpu::Registers;
use cpu::reg;

use mem::Memory;

impl CPU {
  pub fn new() -> CPU {
    CPU {
      regs: Registers::new(),
      m: 0,
      t: 0,
      halt: false,
      stop: false,
    }
  }

  /// Run one instruction.
  /// Increment m and t to account for the time taken by the clock.
  /// Return t, the time taken for this instruction.
  pub fn step(&mut self, mem: &mut Memory) -> u32 {
    // eprintln!(
    //   "0x{:x} 0x{:x} 0x{:x} 0x{:x}",
    //   self.regs.pc,
    //   mem.rb(self.regs.pc),
    //   mem.rb(self.regs.pc + 1),
    //   self.regs.a,
    // );
    let m = self.exec(mem);
    self.regs.m = m;
    self.regs.t = 4 * m;
    self.m += self.regs.m;
    self.t += self.regs.t;
    // eprintln!("TICKS={}", self.regs.t);
    self.regs.t
  }

  /// Execute the next opcode.
  /// Return the m-time taken to run that opcode.
  fn exec(&mut self, mem: &mut Memory) -> u32 {
    macro_rules! bump {
      () => {{
        let result = mem.rb(self.regs.pc);
        self.regs.pc += 1;
        result
      }}
    }
    macro_rules! xx {
      () => {{
        panic!("Invalid opcode");
      }}
    }

    macro_rules! read_u16_le {
      () => {{
        let a = bump!();
        let b = bump!();
        u16::from(a) | (u16::from(b) << 8)
      }}
    }
    macro_rules! ld_nn_n {
      ($reg:ident) => {{
        let imm = bump!();
        self.regs.$reg = imm;
        2
      }}
    }
    macro_rules! ld_n_nn {
      ($r1:ident, $r2:ident) => {{
        self.regs.$r2 = bump!();
        self.regs.$r1 = bump!();
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
        self.regs.$r1 = mem.rb(self.regs.$r2m());
        2
      }}
    }
    macro_rules! ld_r1m_r2 {
      ($r1m:ident, $r2:ident) => {{
        mem.wb(self.regs.$r1m(), self.regs.$r2);
        2
      }}
    }

    macro_rules! push {
      ($r1:ident, $r2:ident) => {{
        mem.wb(self.regs.sp - 1, self.regs.$r1);
        mem.wb(self.regs.sp - 2, self.regs.$r2);
        self.regs.sp -= 2;
        4
      }}
    }
    macro_rules! pop {
      ($r1:ident, $r2:ident) => {{
        self.regs.$r2 = mem.rb(self.regs.sp);
        self.regs.$r1 = mem.rb(self.regs.sp + 1);
        self.regs.sp += 2;
        3
      }}
    }

    macro_rules! add_a_n {
      ($n:expr) => {{
        let a = self.regs.a;
        let n = $n;
        let result = a.wrapping_add(n);
        self.regs.a = result;
        self.regs.f = 0;
        self.regs.f |= if result == 0 {reg::Z} else {0};
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
        let result = a.wrapping_add(n).wrapping_add(c);
        self.regs.a = result;
        self.regs.f = 0;
        self.regs.f |= if result == 0 {reg::Z} else {0};
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
        let result = a.wrapping_sub(n);
        self.regs.a = result;
        self.regs.f = reg::N;
        self.regs.f |= if result == 0 {reg::Z} else {0};
        self.regs.f |= if a < n {reg::C} else {0};
        self.regs.f |= if (a & 0xf) < (n & 0xf) {reg::H} else {0};
        1
      }}
    }
    macro_rules! sbc_a_n {
      ($n:expr) => {{
        let a = self.regs.a as u16;
        let n = $n as u16;
        let c: u16 = if self.regs.c() {1} else {0};
        let result = a.wrapping_sub(n + c);
        self.regs.a = result as u8;
        self.regs.f = reg::N;
        self.regs.f |= if result == 0 {reg::Z} else {0};
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
    macro_rules! xor_a_n {
      ($n:expr) => {{
        self.regs.a ^= $n;
        self.regs.f = if self.regs.a == 0 {reg::Z} else {0};
        1
      }}
    }

    macro_rules! cp_a_n {
      ($n:expr) => {{
        let a = self.regs.a;
        let n = $n;
        self.regs.f = reg::N;
        self.regs.f |= if a == n {reg::Z} else {0};
        self.regs.f |= if (a & 0xf) < (n & 0xf) {reg::H} else {0};
        self.regs.f |= if a < n {reg::C} else {0};
        1
      }}
    }

    macro_rules! inc {
      ($n:expr) => {{
        let n = $n;
        let result = n.wrapping_add(1);
        $n = result;
        let c = self.regs.c();
        self.regs.f = 0;
        self.regs.f |= if result == 0 {reg::Z} else {0};
        self.regs.f |= if n & 0xf == 0xf {reg::H} else {0};
        // Preserve C.
        self.regs.f |= if c {reg::C} else {0};
        1
      }}
    }
    macro_rules! dec {
      ($n:expr) => {{
        let n = $n;
        let result = n.wrapping_sub(1);
        $n = result;
        let c = self.regs.c();
        self.regs.f = reg::N;
        self.regs.f |= if result == 0 {reg::Z} else {0};
        self.regs.f |= if n & 0xf == 0 {reg::H} else {0};
        // Preserve C.
        self.regs.f |= if c {reg::C} else {0};
        1
      }}
    }

    macro_rules! inc_nn {
      ($high:ident, $low:ident) => {{
        let h = self.regs.$high as u16;
        let l = self.regs.$low as u16;
        let n = ((h << 8) | l).wrapping_add(1);
        self.regs.$high = (n >> 8) as u8;
        self.regs.$low = (n & 0x00ff) as u8;
        2
      }}
    }
    macro_rules! dec_nn {
      ($high:ident, $low:ident) => {{
        let h = self.regs.$high as u16;
        let l = self.regs.$low as u16;
        let n = ((h << 8) | l).wrapping_sub(1);
        self.regs.$high = (n >> 8) as u8;
        self.regs.$low = (n & 0x00ff) as u8;
        2
      }}
    }

    macro_rules! add_hl_n {
      ($n:expr) => {{
        let hl = self.regs.hl();
        let n = $n;
        let res = hl.wrapping_add(n);
        let res32 = u32::from(hl) + u32::from(n);
        let z = if self.regs.z() {reg::Z} else {0};
        let n = 0;
        let h = if res32 > 0xfff {reg::H} else {0};
        let c = if res32 > 0xffff {reg::C} else {0};
        self.regs.f = z | n | h | c;
        self.regs.h = ((res & 0xff00) >> 8) as u8;
        self.regs.l = (res & 0xff) as u8;
        2
      }}
    }

    macro_rules! jp {
      () => {{
        self.regs.pc = read_u16_le!();
        4
      }}
    }
    macro_rules! jpc {
      ($e:expr) => {{
        if $e {
          jp!()
        } else {
          self.regs.pc += 2;
          3
        }
      }}
    }

    macro_rules! jr {
      () => {{
        let n = bump!() as i8 as i16;
        // Need to add 1 here to account for bumping past the instruction.
        let pc = self.regs.pc;
        let target = ((pc as i16) + n) as u16;
        self.regs.pc = target;
        3
      }}
    }
    macro_rules! jrc {
      ($e:expr) => {{
        if $e {
          jr!()
        } else {
          self.regs.pc += 1;
          2
        }
      }}
    }


    macro_rules! call {
      () => {{
        let _pc = self.regs.pc;
        self.regs.sp -= 2;
        let target = read_u16_le!();
        let retaddr = self.regs.pc;
        mem.ww(self.regs.sp, retaddr);
        self.regs.pc = target;
        3
      }}
    }
    macro_rules! callc {
      ($e:expr) => {{
        if $e {
          call!()
        } else {
          self.regs.pc += 2;
          3
        }
      }}
    }

    macro_rules! rst {
      ($e:expr) => {{
        self.regs.sp -= 2;
        let retaddr = self.regs.pc;
        mem.ww(self.regs.sp, retaddr);
        self.regs.pc = $e;
        8
      }}
    }

    macro_rules! ret {
      () => {{
        let retaddr = mem.rw(self.regs.sp);
        self.regs.sp += 2;
        self.regs.pc = retaddr;
        4
      }}
    }
    macro_rules! retc {
      ($e:expr) => {{
        if $e {
          ret!()
        } else {
          self.regs.pc += 2;
          2
        }
      }}
    }

    match bump!() {
      0x00 => 1, // nop
      0x01 => ld_n_nn!(b, c),
      0x02 => ld_r1m_r2!(bc, a),
      0x03 => inc_nn!(b, c),
      0x04 => inc!(self.regs.b),
      0x05 => dec!(self.regs.b),
      0x06 => ld_nn_n!(b),
      0x07 => {
        let c = self.regs.a >> 7;
        self.regs.a = (self.regs.a << 1) | c;
        self.regs.f = if self.regs.a == 0 { reg::Z } else { 0 } |
          if c == 1 { reg::C } else { 0 };
        1
      }
      0x08 => {
        let nn = read_u16_le!();
        let val = self.regs.sp;
        mem.ww(nn, val);
        5
      }
      0x09 => add_hl_n!(self.regs.bc()),
      0x0a => ld_r1_r2m!(a, bc),
      0x0b => dec_nn!(b, c),
      0x0c => inc!(self.regs.c),
      0x0d => dec!(self.regs.c),
      0x0e => ld_nn_n!(c),
      0x0f => {
        let c = self.regs.a & 0x1;
        self.regs.a = (self.regs.a >> 1) | (c << 7);
        self.regs.f = if self.regs.a == 0 { reg::Z } else { 0 } |
          if c == 1 { reg::C } else { 0 };
        1
      }

      0x10 => {
        self.stop = true;
        1
      }
      0x11 => ld_n_nn!(d, e),
      0x12 => ld_r1m_r2!(de, a),
      0x13 => inc_nn!(d, e),
      0x14 => inc!(self.regs.d),
      0x15 => dec!(self.regs.d),
      0x16 => ld_nn_n!(d),
      0x17 => {
        let b7 = self.regs.a >> 7;
        let c = if self.regs.c() { 1 } else { 0 };
        self.regs.a = (self.regs.a << 1) | c;
        self.regs.f = if self.regs.a == 0 { reg::Z } else { 0 } |
          if b7 == 1 { reg::C } else { 0 };
        1
      }
      0x18 => jr!(),
      0x19 => add_hl_n!(self.regs.de()),
      0x1a => ld_r1_r2m!(a, de),
      0x1b => dec_nn!(d, e),
      0x1c => inc!(self.regs.e),
      0x1d => dec!(self.regs.e),
      0x1e => ld_nn_n!(e),
      0x1f => {
        let b0 = self.regs.a & 0x1;
        let c = if self.regs.c() { 1 << 7 } else { 0 };
        self.regs.a = (self.regs.a >> 1) | c;
        self.regs.f = if self.regs.a == 0 { reg::Z } else { 0 } |
          if b0 == 1 { reg::C } else { 0 };
        1
      }

      0x20 => jrc!(!self.regs.z()),
      0x21 => ld_n_nn!(h, l),
      0x22 => {
        ld_r1m_r2!(hl, a);
        self.regs.hl_inc();
        2
      }
      0x23 => inc_nn!(h, l),
      0x24 => inc!(self.regs.h),
      0x25 => dec!(self.regs.h),
      0x26 => ld_nn_n!(h),
      0x27 => {
        // DAA
        let mut a = self.regs.a as u16;
        if !self.regs.n() {
          if self.regs.h() || (a & 0xf) > 9 {
            a = a.wrapping_add(0x06);
          }
          if self.regs.c() || a > 0x9f {
            a = a.wrapping_add(0x60);
          }
        } else {
          if self.regs.h() {
            a = a.wrapping_sub(6) & 0xff;
          }
          if self.regs.c() {
            a = a.wrapping_sub(0x60);
          }
        }

        self.regs.f &= !(reg::H | reg::Z);
        if (a & 0x100) == 0x100 {
          self.regs.f |= reg::C;
        }

        a &= 0xff;

        if a == 0 {
          self.regs.f |= reg::Z;
        }

        self.regs.a = a as u8;
        1
      }
      0x28 => jrc!(self.regs.z()),
      0x29 => add_hl_n!(self.regs.hl()),
      0x2a => {
        ld_r1_r2m!(a, hl);
        self.regs.hl_inc();
        2
      }
      0x2b => dec_nn!(h, l),
      0x2c => inc!(self.regs.l),
      0x2d => dec!(self.regs.l),
      0x2e => ld_nn_n!(l),
      0x2f => {
        self.regs.a = !self.regs.a;
        let f = self.regs.f;
        self.regs.f = (f & reg::Z) | reg::N | reg::H | (f & reg::C);
        1
      }

      0x30 => jrc!(!self.regs.c()),
      0x31 => {
        self.regs.sp = read_u16_le!();
        3
      }
      0x32 => {
        ld_r1m_r2!(hl, a);
        self.regs.hl_dec();
        2
      }
      0x33 => {
        self.regs.sp += 1;
        2
      }
      0x34 => {
        let n = mem.rb(self.regs.hl());
        let c = if self.regs.c() { reg::C } else { 0 };
        mem.wb(self.regs.hl(), n + 1);
        self.regs.f = if n + 1 == 0 { reg::Z } else { 0 } |
          if n & 0xf == 0xf { reg::H } else { 0 } | c;
        3
      }
      0x35 => {
        let n = mem.rb(self.regs.hl());
        let c = if self.regs.c() { reg::C } else { 0 };
        let result = n.wrapping_sub(1);
        mem.wb(self.regs.hl(), result);
        self.regs.f = if result == 0 { reg::Z } else { 0 } |
          if n & 0xf != 0 { reg::H } else { 0 } | c;
        3
      }
      0x36 => {
        let n = bump!();
        mem.wb(self.regs.hl(), n);
        3
      }
      0x37 => {
        self.regs.f = (self.regs.f & reg::Z) | reg::C;
        1
      }
      0x38 => jrc!(self.regs.c()),
      0x39 => add_hl_n!(self.regs.sp),
      0x3a => {
        ld_r1_r2m!(a, hl);
        self.regs.hl_dec();
        2
      }
      0x3b => {
        self.regs.sp -= 1;
        2
      }
      0x3c => inc!(self.regs.a),
      0x3d => dec!(self.regs.a),
      0x3e => {
        self.regs.a = bump!();
        2
      }
      0x3f => {
        self.regs.f = (self.regs.f & reg::Z) |
          if self.regs.c() { 0 } else { reg::C };
        1
      }

      0x40 => ld_r1_r2!(b, b),
      0x41 => ld_r1_r2!(b, c),
      0x42 => ld_r1_r2!(b, d),
      0x43 => ld_r1_r2!(b, e),
      0x44 => ld_r1_r2!(b, h),
      0x45 => ld_r1_r2!(b, l),
      0x46 => ld_r1_r2m!(b, hl),
      0x47 => ld_r1_r2!(b, a),
      0x48 => ld_r1_r2!(c, b),
      0x49 => ld_r1_r2!(c, c),
      0x4a => ld_r1_r2!(c, d),
      0x4b => ld_r1_r2!(c, e),
      0x4c => ld_r1_r2!(c, h),
      0x4d => ld_r1_r2!(c, l),
      0x4e => ld_r1_r2m!(c, hl),
      0x4f => ld_r1_r2!(c, a),

      0x50 => ld_r1_r2!(d, b),
      0x51 => ld_r1_r2!(d, c),
      0x52 => ld_r1_r2!(d, d),
      0x53 => ld_r1_r2!(d, e),
      0x54 => ld_r1_r2!(d, h),
      0x55 => ld_r1_r2!(d, l),
      0x56 => ld_r1_r2m!(d, hl),
      0x57 => ld_r1_r2!(d, a),
      0x58 => ld_r1_r2!(e, b),
      0x59 => ld_r1_r2!(e, c),
      0x5a => ld_r1_r2!(e, d),
      0x5b => ld_r1_r2!(e, e),
      0x5c => ld_r1_r2!(e, h),
      0x5d => ld_r1_r2!(e, l),
      0x5e => ld_r1_r2m!(e, hl),
      0x5f => ld_r1_r2!(e, a),

      0x60 => ld_r1_r2!(h, b),
      0x61 => ld_r1_r2!(h, c),
      0x62 => ld_r1_r2!(h, d),
      0x63 => ld_r1_r2!(h, e),
      0x64 => ld_r1_r2!(h, h),
      0x65 => ld_r1_r2!(h, l),
      0x66 => ld_r1_r2m!(h, hl),
      0x67 => ld_r1_r2!(h, a),
      0x68 => ld_r1_r2!(l, b),
      0x69 => ld_r1_r2!(l, c),
      0x6a => ld_r1_r2!(l, d),
      0x6b => ld_r1_r2!(l, e),
      0x6c => ld_r1_r2!(l, h),
      0x6d => ld_r1_r2!(l, l),
      0x6e => ld_r1_r2m!(l, hl),
      0x6f => ld_r1_r2!(l, a),

      0x70 => ld_r1m_r2!(hl, b),
      0x71 => ld_r1m_r2!(hl, c),
      0x72 => ld_r1m_r2!(hl, d),
      0x73 => ld_r1m_r2!(hl, e),
      0x74 => ld_r1m_r2!(hl, h),
      0x75 => ld_r1m_r2!(hl, l),
      0x76 => {
        self.halt = true;
        1
      }
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
      0x86 => {
        add_a_n!(mem.rb(self.regs.hl()));
        2
      }
      0x87 => add_a_n!(self.regs.a),
      0x88 => adc_a_n!(self.regs.b),
      0x89 => adc_a_n!(self.regs.c),
      0x8a => adc_a_n!(self.regs.d),
      0x8b => adc_a_n!(self.regs.e),
      0x8c => adc_a_n!(self.regs.h),
      0x8d => adc_a_n!(self.regs.l),
      0x8e => {
        adc_a_n!(mem.rb(self.regs.hl()));
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
        sub_a_n!(mem.rb(self.regs.hl()));
        2
      }
      0x97 => sub_a_n!(self.regs.a),
      0x98 => sbc_a_n!(self.regs.b),
      0x99 => sbc_a_n!(self.regs.c),
      0x9a => sbc_a_n!(self.regs.d),
      0x9b => sbc_a_n!(self.regs.e),
      0x9c => sbc_a_n!(self.regs.h),
      0x9d => sbc_a_n!(self.regs.l),
      0x9e => {
        sbc_a_n!(mem.rb(self.regs.hl()));
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
        and_a_n!(mem.rb(self.regs.hl()));
        2
      }
      0xa7 => and_a_n!(self.regs.a),
      0xa8 => xor_a_n!(self.regs.b),
      0xa9 => xor_a_n!(self.regs.c),
      0xaa => xor_a_n!(self.regs.d),
      0xab => xor_a_n!(self.regs.e),
      0xac => xor_a_n!(self.regs.h),
      0xad => xor_a_n!(self.regs.l),
      0xae => {
        xor_a_n!(mem.rb(self.regs.hl()));
        2
      }
      0xaf => xor_a_n!(self.regs.a),

      0xb0 => or_a_n!(self.regs.b),
      0xb1 => or_a_n!(self.regs.c),
      0xb2 => or_a_n!(self.regs.d),
      0xb3 => or_a_n!(self.regs.e),
      0xb4 => or_a_n!(self.regs.h),
      0xb5 => or_a_n!(self.regs.l),
      0xb6 => {
        or_a_n!(mem.rb(self.regs.hl()));
        2
      }
      0xb7 => or_a_n!(self.regs.a),
      0xb8 => cp_a_n!(self.regs.b),
      0xb9 => cp_a_n!(self.regs.c),
      0xba => cp_a_n!(self.regs.d),
      0xbb => cp_a_n!(self.regs.e),
      0xbc => cp_a_n!(self.regs.h),
      0xbd => cp_a_n!(self.regs.l),
      0xbe => {
        cp_a_n!(mem.rb(self.regs.hl()));
        2
      }
      0xbf => cp_a_n!(self.regs.a),

      0xc0 => retc!(!self.regs.z()),
      0xc1 => pop!(b, c),
      0xc2 => jpc!(!self.regs.z()),
      0xc3 => jp!(),
      0xc4 => callc!(!self.regs.z()),
      0xc5 => push!(b, c),
      0xc6 => {
        let n = bump!();
        add_a_n!(n);
        2
      }
      0xc7 => rst!(0x00),
      0xc8 => retc!(self.regs.z()),
      0xc9 => ret!(),
      0xca => {
        if self.regs.z() {
          self.regs.pc = read_u16_le!();
        }
        3
      }
      0xcb => self.exec_cb(mem),
      0xcc => callc!(self.regs.z()),
      0xcd => {
        call!();
        6
      }
      0xce => {
        let n = bump!();
        adc_a_n!(n);
        2
      }
      0xcf => rst!(0x08),

      0xd0 => retc!(!self.regs.c()),
      0xd1 => pop!(d, e),
      0xd2 => jpc!(!self.regs.c()),
      0xd3 => xx!(),
      0xd4 => callc!(!self.regs.c()),
      0xd5 => push!(d, e),
      0xd6 => {
        let n = bump!();
        sub_a_n!(n);
        2
      }
      0xd7 => rst!(0x10),
      0xd8 => retc!(self.regs.c()),
      0xd9 => {
        // TODO: Enable interrupts.
        ret!()
      }
      0xda => jpc!(self.regs.c()),
      0xdb => unimplemented!(),
      0xdc => callc!(self.regs.c()),
      0xdd => unimplemented!(),
      0xde => {
        let n = bump!();
        sbc_a_n!(n);
        2
      }
      0xdf => rst!(0x18),

      0xe0 => {
        let n = bump!();
        mem.wb(0xff00 + u16::from(n), self.regs.a);
        3
      }
      0xe1 => pop!(h, l),
      0xe2 => {
        mem.wb(0xff00 + u16::from(self.regs.c), self.regs.a);
        2
      }
      0xe3 => unimplemented!(),
      0xe4 => unimplemented!(),
      0xe5 => push!(h, l),
      0xe6 => {
        let n = bump!();
        and_a_n!(n);
        2
      }
      0xe7 => rst!(0x20),
      0xe8 => {
        let sp = self.regs.sp;
        // Add a signed 8-bit immediate to sp.
        // This requires casting to i8, sign extending, and then back to u16.
        let n = bump!() as i8 as i16;
        let res = ((sp as i16) + n) as u16;
        // Sleight-of-hand to check the carry and half-carry flags
        // and handle both negative and non-negative cases elegantly.
        // Essentially spooky bit twiddling.
        let tmp = (n as u16) ^ res ^ sp;
        self.regs.f = if tmp & 0x100 != 0 { reg::C } else { 0 } |
          if tmp & 0x010 != 0 { reg::H } else { 0 };
        self.regs.sp = res;
        4
      }
      0xe9 => {
        self.regs.pc = self.regs.hl();
        1
      }
      0xea => {
        let nn = read_u16_le!();
        mem.wb(nn, self.regs.a);
        4
      }
      0xeb => unimplemented!(),
      0xec => unimplemented!(),
      0xed => unimplemented!(),
      0xee => {
        let n = bump!();
        xor_a_n!(n);
        2
      }
      0xef => rst!(0x28),

      0xf0 => {
        let n = bump!();
        self.regs.a = mem.rb(0xff00 + (n as u16));
        3
      }
      0xf1 => {
        // pop af
        self.regs.f = mem.rb(self.regs.sp) & 0xf0;
        self.regs.a = mem.rb(self.regs.sp + 1);
        self.regs.sp += 2;
        3
      }
      0xf2 => {
        self.regs.a = mem.rb(0xff00 + (self.regs.c as u16));
        2
      }
      0xf3 => {
        // TODO: Disable interrupts.
        1
      }
      0xf4 => xx!(),
      0xf5 => push!(a, f),
      0xf6 => {
        let n = bump!();
        or_a_n!(n);
        2
      }
      0xf7 => rst!(0x30),
      0xf8 => {
        let n = bump!() as i8 as i16;
        let sp = self.regs.sp;
        let res = ((sp as i16) + n) as u16;
        self.regs.h = (res >> 8) as u8;
        self.regs.l = (res & 0xf) as u8;
        let tmp = (n as u16) ^ res ^ sp;
        self.regs.f = if tmp & 0x100 != 0 { reg::C } else { 0 } |
          if tmp & 0x010 != 0 { reg::H } else { 0 };
        3
      }
      0xf9 => {
        self.regs.sp = self.regs.hl();
        2
      }
      0xfa => {
        let nn = read_u16_le!();
        self.regs.a = mem.rb(nn);
        4
      }
      0xfb => {
        // TODO: Disable interrupts.
        1
      }
      0xfc => xx!(),
      0xfd => xx!(),
      0xfe => {
        let n = bump!();
        cp_a_n!(n);
        2
      }
      0xff => rst!(0x38),
      _ => panic!("Invalid opcode"),
    }
  }

  /// Run cb instruction.
  fn exec_cb(&mut self, mem: &mut Memory) -> u32 {
    macro_rules! bump {
      () => {{
        let result = mem.rb(self.regs.pc);
        self.regs.pc += 1;
        result
      }}
    }

    macro_rules! do_hl {
      ($int:ident, $stmt:stmt, $time:expr) => {{
        let mut $int = mem.rb(self.regs.hl());
        $stmt;
        mem.wb(self.regs.hl(), $int);
        $time as u32
      }}
    }

    macro_rules! rlc {
      ($reg:expr, $time:expr) => {{
        let c = $reg >> 7;
        $reg = ($reg << 1) | c;
        self.regs.f = if $reg == 0 { reg::Z } else { 0 } |
          if c == 1 { reg::C } else { 0 };
        $time as u32
      }}
    }
    macro_rules! rl {
      ($reg:expr, $time:expr) => {{
        let b7 = $reg >> 7;
        let c = if self.regs.c() { 1 } else { 0 };
        $reg = ($reg << 1) | c;
        self.regs.f = if $reg == 0 { reg::Z } else { 0 } |
          if b7 == 1 { reg::C } else { 0 };
        $time as u32
      }}
    }

    macro_rules! rrc {
      ($reg:expr, $time:expr) => {{
        let c = $reg & 0x1;
        $reg = ($reg >> 1) | (c << 7);
        self.regs.f = if $reg == 0 { reg::Z } else { 0 } |
          if c == 1 { reg::C } else { 0 };
        $time as u32
      }}
    }
    macro_rules! rr {
      ($reg:expr, $time:expr) => {{
        let b0 = $reg & 0x1;
        let c = if self.regs.c() { 1 } else { 0 };
        $reg = ($reg >> 1) | (c << 7);
        self.regs.f = if $reg == 0 { reg::Z } else { 0 } |
          if b0 == 1 { reg::C } else { 0 };
        $time as u32
      }}
    }

    macro_rules! sla {
      ($reg:expr, $time:expr) => {{
        let b7 = $reg >> 7;
        $reg <<= 1;
        self.regs.f = if $reg == 0 {reg::Z} else {0} |
          if b7 == 1 { reg::C } else { 0 };
        $time as u32
      }}
    }
    macro_rules! sra {
      ($reg:expr, $time:expr) => {{
        let b0 = $reg & 0x1;
        // Sign extend.
        $reg = (($reg as i8) >> 1) as u8;
        self.regs.f = if $reg == 0 {reg::Z} else {0} |
          if b0 == 1 { reg::C } else { 0 };
        $time as u32
      }}
    }
    macro_rules! srl {
      ($reg:expr, $time:expr) => {{
        let b0 = $reg & 0x1;
        $reg = ($reg as u8) >> 1;
        self.regs.f = if $reg == 0 {reg::Z} else {0} |
          if b0 == 1 { reg::C } else { 0 };
        $time as u32
      }}
    }

    macro_rules! bit {
      ($reg:expr, $b:expr, $time:expr) => {{
        let b = (1 << $b) & $reg;
        self.regs.f = reg::H | if b == 0 {reg::Z} else {0};
        $time as u32
      }}
    }
    macro_rules! set {
      ($reg:expr, $b:expr, $time:expr) => {{
        $reg = (1 << $b) | $reg;
        $time as u32
      }}
    }
    macro_rules! reset {
      ($reg:expr, $b:expr, $time:expr) => {{
        $reg = !(1 << $b) & $reg;
        $time as u32
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

    match bump!() {
      0x00 => rlc!(self.regs.b, 2),
      0x01 => rlc!(self.regs.c, 2),
      0x02 => rlc!(self.regs.d, 2),
      0x03 => rlc!(self.regs.e, 2),
      0x04 => rlc!(self.regs.h, 2),
      0x05 => rlc!(self.regs.l, 2),
      0x06 => do_hl!(hl, rlc!(hl, 1), 4),
      0x07 => rlc!(self.regs.a, 2),
      0x08 => rrc!(self.regs.b, 2),
      0x09 => rrc!(self.regs.c, 2),
      0x0a => rrc!(self.regs.d, 2),
      0x0b => rrc!(self.regs.e, 2),
      0x0c => rrc!(self.regs.h, 2),
      0x0d => rrc!(self.regs.l, 2),
      0x0e => do_hl!(hl, rrc!(hl, 1), 4),
      0x0f => rrc!(self.regs.a, 2),

      0x10 => rl!(self.regs.b, 2),
      0x11 => rl!(self.regs.c, 2),
      0x12 => rl!(self.regs.d, 2),
      0x13 => rl!(self.regs.e, 2),
      0x14 => rl!(self.regs.h, 2),
      0x15 => rl!(self.regs.l, 2),
      0x16 => do_hl!(hl, rl!(hl, 1), 4),
      0x17 => rl!(self.regs.a, 2),
      0x18 => rr!(self.regs.b, 2),
      0x19 => rr!(self.regs.c, 2),
      0x1a => rr!(self.regs.d, 2),
      0x1b => rr!(self.regs.e, 2),
      0x1c => rr!(self.regs.h, 2),
      0x1d => rr!(self.regs.l, 2),
      0x1e => do_hl!(hl, rr!(hl, 1), 4),
      0x1f => rr!(self.regs.a, 2),

      0x20 => sla!(self.regs.b, 2),
      0x21 => sla!(self.regs.c, 2),
      0x22 => sla!(self.regs.d, 2),
      0x23 => sla!(self.regs.e, 2),
      0x24 => sla!(self.regs.h, 2),
      0x25 => sla!(self.regs.l, 2),
      0x26 => do_hl!(hl, sla!(hl, 1), 4),
      0x27 => sla!(self.regs.a, 2),
      0x28 => sra!(self.regs.b, 2),
      0x29 => sra!(self.regs.c, 2),
      0x2a => sra!(self.regs.d, 2),
      0x2b => sra!(self.regs.e, 2),
      0x2c => sra!(self.regs.h, 2),
      0x2d => sra!(self.regs.l, 2),
      0x2e => do_hl!(hl, sra!(hl, 1), 4),
      0x2f => sra!(self.regs.a, 2),

      0x30 => swap!(b),
      0x31 => swap!(c),
      0x32 => swap!(d),
      0x33 => swap!(e),
      0x34 => swap!(h),
      0x35 => swap!(l),
      0x36 => {
        let byte = mem.rb(self.regs.hl());
        let top = byte >> 4;
        let bot = byte & 0xf;
        mem.wb(self.regs.hl(), (bot << 4) | top);
        4
      }
      0x37 => swap!(a),
      0x38 => srl!(self.regs.b, 2),
      0x39 => srl!(self.regs.c, 2),
      0x3a => srl!(self.regs.d, 2),
      0x3b => srl!(self.regs.e, 2),
      0x3c => srl!(self.regs.h, 2),
      0x3d => srl!(self.regs.l, 2),
      0x3e => do_hl!(hl, srl!(hl, 1), 4),
      0x3f => srl!(self.regs.a, 2),

      0x40 => bit!(self.regs.b, 0, 2),
      0x41 => bit!(self.regs.c, 0, 2),
      0x42 => bit!(self.regs.d, 0, 2),
      0x43 => bit!(self.regs.e, 0, 2),
      0x44 => bit!(self.regs.h, 0, 2),
      0x45 => bit!(self.regs.l, 0, 2),
      0x46 => bit!(mem.rb(self.regs.hl()), 0, 4),
      0x47 => bit!(self.regs.a, 0, 2),
      0x48 => bit!(self.regs.b, 1, 2),
      0x49 => bit!(self.regs.c, 1, 2),
      0x4a => bit!(self.regs.d, 1, 2),
      0x4b => bit!(self.regs.e, 1, 2),
      0x4c => bit!(self.regs.h, 1, 2),
      0x4d => bit!(self.regs.l, 1, 2),
      0x4e => bit!(mem.rb(self.regs.hl()), 1, 4),
      0x4f => bit!(self.regs.a, 1, 2),

      0x50 => bit!(self.regs.b, 2, 2),
      0x51 => bit!(self.regs.c, 2, 2),
      0x52 => bit!(self.regs.d, 2, 2),
      0x53 => bit!(self.regs.e, 2, 2),
      0x54 => bit!(self.regs.h, 2, 2),
      0x55 => bit!(self.regs.l, 2, 2),
      0x56 => bit!(mem.rb(self.regs.hl()), 2, 4),
      0x57 => bit!(self.regs.a, 2, 2),
      0x58 => bit!(self.regs.b, 3, 2),
      0x59 => bit!(self.regs.c, 3, 2),
      0x5a => bit!(self.regs.d, 3, 2),
      0x5b => bit!(self.regs.e, 3, 2),
      0x5c => bit!(self.regs.h, 3, 2),
      0x5d => bit!(self.regs.l, 3, 2),
      0x5e => bit!(mem.rb(self.regs.hl()), 3, 4),
      0x5f => bit!(self.regs.a, 3, 2),

      0x60 => bit!(self.regs.b, 4, 2),
      0x61 => bit!(self.regs.c, 4, 2),
      0x62 => bit!(self.regs.d, 4, 2),
      0x63 => bit!(self.regs.e, 4, 2),
      0x64 => bit!(self.regs.h, 4, 2),
      0x65 => bit!(self.regs.l, 4, 2),
      0x66 => bit!(mem.rb(self.regs.hl()), 4, 4),
      0x67 => bit!(self.regs.a, 4, 2),
      0x68 => bit!(self.regs.b, 5, 2),
      0x69 => bit!(self.regs.c, 5, 2),
      0x6a => bit!(self.regs.d, 5, 2),
      0x6b => bit!(self.regs.e, 5, 2),
      0x6c => bit!(self.regs.h, 5, 2),
      0x6d => bit!(self.regs.l, 5, 2),
      0x6e => bit!(mem.rb(self.regs.hl()), 5, 4),
      0x6f => bit!(self.regs.a, 5, 2),

      0x70 => bit!(self.regs.b, 6, 2),
      0x71 => bit!(self.regs.c, 6, 2),
      0x72 => bit!(self.regs.d, 6, 2),
      0x73 => bit!(self.regs.e, 6, 2),
      0x74 => bit!(self.regs.h, 6, 2),
      0x75 => bit!(self.regs.l, 6, 2),
      0x76 => bit!(mem.rb(self.regs.hl()), 6, 4),
      0x77 => bit!(self.regs.a, 6, 2),
      0x78 => bit!(self.regs.b, 7, 2),
      0x79 => bit!(self.regs.c, 7, 2),
      0x7a => bit!(self.regs.d, 7, 2),
      0x7b => bit!(self.regs.e, 7, 2),
      0x7c => bit!(self.regs.h, 7, 2),
      0x7d => bit!(self.regs.l, 7, 2),
      0x7e => bit!(mem.rb(self.regs.hl()), 7, 4),
      0x7f => bit!(self.regs.a, 7, 2),

      0x80 => reset!(self.regs.b, 0, 2),
      0x81 => reset!(self.regs.c, 0, 2),
      0x82 => reset!(self.regs.d, 0, 2),
      0x83 => reset!(self.regs.e, 0, 2),
      0x84 => reset!(self.regs.h, 0, 2),
      0x85 => reset!(self.regs.l, 0, 2),
      0x86 => do_hl!(hl, reset!(hl, 0, 1), 4),
      0x87 => reset!(self.regs.a, 0, 2),
      0x88 => reset!(self.regs.b, 1, 2),
      0x89 => reset!(self.regs.c, 1, 2),
      0x8a => reset!(self.regs.d, 1, 2),
      0x8b => reset!(self.regs.e, 1, 2),
      0x8c => reset!(self.regs.h, 1, 2),
      0x8d => reset!(self.regs.l, 1, 2),
      0x8e => do_hl!(hl, reset!(hl, 1, 1), 4),
      0x8f => reset!(self.regs.a, 1, 2),

      0x90 => reset!(self.regs.b, 2, 2),
      0x91 => reset!(self.regs.c, 2, 2),
      0x92 => reset!(self.regs.d, 2, 2),
      0x93 => reset!(self.regs.e, 2, 2),
      0x94 => reset!(self.regs.h, 2, 2),
      0x95 => reset!(self.regs.l, 2, 2),
      0x96 => do_hl!(hl, reset!(hl, 2, 1), 4),
      0x97 => reset!(self.regs.a, 2, 2),
      0x98 => reset!(self.regs.b, 3, 2),
      0x99 => reset!(self.regs.c, 3, 2),
      0x9a => reset!(self.regs.d, 3, 2),
      0x9b => reset!(self.regs.e, 3, 2),
      0x9c => reset!(self.regs.h, 3, 2),
      0x9d => reset!(self.regs.l, 3, 2),
      0x9e => do_hl!(hl, reset!(hl, 3, 1), 4),
      0x9f => reset!(self.regs.a, 3, 2),

      0xa0 => reset!(self.regs.b, 4, 2),
      0xa1 => reset!(self.regs.c, 4, 2),
      0xa2 => reset!(self.regs.d, 4, 2),
      0xa3 => reset!(self.regs.e, 4, 2),
      0xa4 => reset!(self.regs.h, 4, 2),
      0xa5 => reset!(self.regs.l, 4, 2),
      0xa6 => do_hl!(hl, reset!(hl, 4, 1), 4),
      0xa7 => reset!(self.regs.a, 4, 2),
      0xa8 => reset!(self.regs.b, 5, 2),
      0xa9 => reset!(self.regs.c, 5, 2),
      0xaa => reset!(self.regs.d, 5, 2),
      0xab => reset!(self.regs.e, 5, 2),
      0xac => reset!(self.regs.h, 5, 2),
      0xad => reset!(self.regs.l, 5, 2),
      0xae => do_hl!(hl, reset!(hl, 5, 1), 4),
      0xaf => reset!(self.regs.a, 5, 2),

      0xb0 => reset!(self.regs.b, 6, 2),
      0xb1 => reset!(self.regs.c, 6, 2),
      0xb2 => reset!(self.regs.d, 6, 2),
      0xb3 => reset!(self.regs.e, 6, 2),
      0xb4 => reset!(self.regs.h, 6, 2),
      0xb5 => reset!(self.regs.l, 6, 2),
      0xb6 => do_hl!(hl, reset!(hl, 6, 1), 4),
      0xb7 => reset!(self.regs.a, 6, 2),
      0xb8 => reset!(self.regs.b, 7, 2),
      0xb9 => reset!(self.regs.c, 7, 2),
      0xba => reset!(self.regs.d, 7, 2),
      0xbb => reset!(self.regs.e, 7, 2),
      0xbc => reset!(self.regs.h, 7, 2),
      0xbd => reset!(self.regs.l, 7, 2),
      0xbe => do_hl!(hl, reset!(hl, 7, 1), 4),
      0xbf => reset!(self.regs.a, 7, 2),

      0xc0 => set!(self.regs.b, 0, 2),
      0xc1 => set!(self.regs.c, 0, 2),
      0xc2 => set!(self.regs.d, 0, 2),
      0xc3 => set!(self.regs.e, 0, 2),
      0xc4 => set!(self.regs.h, 0, 2),
      0xc5 => set!(self.regs.l, 0, 2),
      0xc6 => do_hl!(hl, set!(hl, 0, 1), 4),
      0xc7 => set!(self.regs.a, 0, 2),
      0xc8 => set!(self.regs.b, 1, 2),
      0xc9 => set!(self.regs.c, 1, 2),
      0xca => set!(self.regs.d, 1, 2),
      0xcb => set!(self.regs.e, 1, 2),
      0xcc => set!(self.regs.h, 1, 2),
      0xcd => set!(self.regs.l, 1, 2),
      0xce => do_hl!(hl, set!(hl, 1, 1), 4),
      0xcf => set!(self.regs.a, 1, 2),

      0xd0 => set!(self.regs.b, 2, 2),
      0xd1 => set!(self.regs.c, 2, 2),
      0xd2 => set!(self.regs.d, 2, 2),
      0xd3 => set!(self.regs.e, 2, 2),
      0xd4 => set!(self.regs.h, 2, 2),
      0xd5 => set!(self.regs.l, 2, 2),
      0xd6 => do_hl!(hl, set!(hl, 2, 1), 4),
      0xd7 => set!(self.regs.a, 2, 2),
      0xd8 => set!(self.regs.b, 3, 2),
      0xd9 => set!(self.regs.c, 3, 2),
      0xda => set!(self.regs.d, 3, 2),
      0xdb => set!(self.regs.e, 3, 2),
      0xdc => set!(self.regs.h, 3, 2),
      0xdd => set!(self.regs.l, 3, 2),
      0xde => do_hl!(hl, set!(hl, 3, 1), 4),
      0xdf => set!(self.regs.a, 3, 2),

      0xe0 => set!(self.regs.b, 4, 2),
      0xe1 => set!(self.regs.c, 4, 2),
      0xe2 => set!(self.regs.d, 4, 2),
      0xe3 => set!(self.regs.e, 4, 2),
      0xe4 => set!(self.regs.h, 4, 2),
      0xe5 => set!(self.regs.l, 4, 2),
      0xe6 => do_hl!(hl, set!(hl, 4, 1), 4),
      0xe7 => set!(self.regs.a, 4, 2),
      0xe8 => set!(self.regs.b, 5, 2),
      0xe9 => set!(self.regs.c, 5, 2),
      0xea => set!(self.regs.d, 5, 2),
      0xeb => set!(self.regs.e, 5, 2),
      0xec => set!(self.regs.h, 5, 2),
      0xed => set!(self.regs.l, 5, 2),
      0xee => do_hl!(hl, set!(hl, 5, 1), 4),
      0xef => set!(self.regs.a, 5, 2),

      0xf0 => set!(self.regs.b, 6, 2),
      0xf1 => set!(self.regs.c, 6, 2),
      0xf2 => set!(self.regs.d, 6, 2),
      0xf3 => set!(self.regs.e, 6, 2),
      0xf4 => set!(self.regs.h, 6, 2),
      0xf5 => set!(self.regs.l, 6, 2),
      0xf6 => do_hl!(hl, set!(hl, 6, 1), 4),
      0xf7 => set!(self.regs.a, 6, 2),
      0xf8 => set!(self.regs.b, 7, 2),
      0xf9 => set!(self.regs.c, 7, 2),
      0xfa => set!(self.regs.d, 7, 2),
      0xfb => set!(self.regs.e, 7, 2),
      0xfc => set!(self.regs.h, 7, 2),
      0xfd => set!(self.regs.l, 7, 2),
      0xfe => do_hl!(hl, set!(hl, 7, 1), 4),
      0xff => set!(self.regs.a, 7, 2),

      _ => panic!("Inexhaustive match"),
    }
  }
}
