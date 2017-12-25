use cpu::CPU;
use mem::Memory;

fn init() -> (CPU, Memory) {
  let mut cpu = CPU::new();
  let mem = Memory::new();
  // Se the PC to start in WRAM.
  cpu.regs.pc = 0xe000;
  (cpu, mem)
}

fn run(
  cpu: &mut CPU,
  mem: &mut Memory,
  opcode: u8,
  len: u16,
  time_expected: u32,
) {
  let start = cpu.regs.pc;
  mem.wb(cpu.regs.pc, opcode);
  cpu.step(mem);
  let time_actual = cpu.regs.m;
  // Test time.
  assert_eq!(time_actual, time_expected);
  // Test that the PC was incremented.
  assert_eq!(cpu.regs.pc, start + len);
}

#[test]
fn nop() {
  let (mut cpu, mut mem) = init();
  run(&mut cpu, &mut mem, 0x00, 1, 1);
}

#[test]
fn ld_nn_n() {
  macro_rules! run_test {
    ($reg:ident, $opcode:expr) => {{
      let (mut cpu, mut mem) = init();
      mem.wb(cpu.regs.pc + 1, 0x42);
      let f = cpu.regs.f;
      run(&mut cpu, &mut mem, $opcode, 2, 2);
      assert_eq!(cpu.regs.f, f);
      assert_eq!(cpu.regs.$reg, 0x42);
    }}
  }
  run_test!(b, 0x06);
  run_test!(c, 0x0e);
  run_test!(d, 0x16);
  run_test!(e, 0x1e);
  run_test!(h, 0x26);
  run_test!(l, 0x2e);
}

#[test]
fn ld_r1_r2() {
  macro_rules! reg_reg {
      ($r1:ident, $r2:ident, $opcode:expr) => {{
        let (mut cpu, mut mem) = init();
        cpu.regs.$r2 = 0x42;
        let f = cpu.regs.f;
        run(&mut cpu, &mut mem, $opcode, 1, 1);
        assert_eq!(cpu.regs.f, f);
        assert_eq!(cpu.regs.$r1, 0x42);
        assert_eq!(cpu.regs.$r2, 0x42);
      }}
    }

  reg_reg!(a, a, 0x7f);
  reg_reg!(a, b, 0x78);
  reg_reg!(a, c, 0x79);
  reg_reg!(a, d, 0x7a);
  reg_reg!(a, e, 0x7b);
  reg_reg!(a, h, 0x7c);
  reg_reg!(a, l, 0x7d);

  reg_reg!(b, b, 0x40);
  reg_reg!(b, c, 0x41);
  reg_reg!(b, d, 0x42);
  reg_reg!(b, e, 0x43);
  reg_reg!(b, h, 0x44);
  reg_reg!(b, l, 0x45);

  reg_reg!(c, b, 0x48);
  reg_reg!(c, c, 0x49);
  reg_reg!(c, d, 0x4a);
  reg_reg!(c, e, 0x4b);
  reg_reg!(c, h, 0x4c);
  reg_reg!(c, l, 0x4d);

  reg_reg!(d, b, 0x50);
  reg_reg!(d, c, 0x51);
  reg_reg!(d, d, 0x52);
  reg_reg!(d, e, 0x53);
  reg_reg!(d, h, 0x54);
  reg_reg!(d, l, 0x55);

  reg_reg!(e, b, 0x58);
  reg_reg!(e, c, 0x59);
  reg_reg!(e, d, 0x5a);
  reg_reg!(e, e, 0x5b);
  reg_reg!(e, h, 0x5c);
  reg_reg!(e, l, 0x5d);

  reg_reg!(h, b, 0x60);
  reg_reg!(h, c, 0x61);
  reg_reg!(h, d, 0x62);
  reg_reg!(h, e, 0x63);
  reg_reg!(h, h, 0x64);
  reg_reg!(h, l, 0x65);

  reg_reg!(l, b, 0x68);
  reg_reg!(l, c, 0x69);
  reg_reg!(l, d, 0x6a);
  reg_reg!(l, e, 0x6b);
  reg_reg!(l, h, 0x6c);
  reg_reg!(l, l, 0x6d);

  // TODO: Add tests for (HL) loads/stores.
}
