use crate::chip8::Chip8;

#[test]
fn chip8_jmp_addr() {
    let mut chip8 = Chip8::new();
    chip8.exec_instr(0x1555);
    assert_eq!(chip8.regs.pc, 0x555);
}

#[test]
fn chip8_call_ret() {
    let mut chip8 = Chip8::new();
    chip8.exec_instr(0x2555);
    assert_eq!(chip8.regs.pc, 0x555);
    assert_eq!(chip8.regs.sp, 1);
    assert_eq!(chip8.regs.stack[0], 0x202);
    chip8.exec_instr(0x2777);
    assert_eq!(chip8.regs.pc, 0x777);
    assert_eq!(chip8.regs.sp, 2);
    assert_eq!(chip8.regs.stack[1], 0x557);
    chip8.exec_instr(0x00EE);
    assert_eq!(chip8.regs.pc, 0x557);
    assert_eq!(chip8.regs.sp, 1);
    chip8.exec_instr(0x00EE);
    assert_eq!(chip8.regs.pc, 0x202);
    assert_eq!(chip8.regs.sp, 0);
}

#[test]
#[should_panic]
fn chip8_call_stack_overflow() {
    let mut chip8 = Chip8::new();
    chip8.regs.sp = 15;
    chip8.exec_instr(0x2555);
}

#[test]
#[should_panic]
fn chip8_ret_stack_overflow() {
    let mut chip8 = Chip8::new();
    chip8.regs.sp = 0;
    chip8.exec_instr(0x00EE);
}

#[test]
fn chip8_skip_instr() {
    let mut chip8 = Chip8::new();
    assert_eq!(chip8.regs.pc, 0x200);
    chip8.exec_instr(0x3455);
    assert_eq!(chip8.regs.pc, 0x202);
    chip8.regs.v[4] = 0x55;
    chip8.exec_instr(0x3455);
    assert_eq!(chip8.regs.pc, 0x206);
    chip8.exec_instr(0x4400);
    assert_eq!(chip8.regs.pc, 0x20A);
    chip8.exec_instr(0x4455);
    assert_eq!(chip8.regs.pc, 0x20C);
    chip8.exec_instr(0x5450);
    assert_eq!(chip8.regs.pc, 0x20E);
    chip8.regs.v[5] = 0x55;
    chip8.exec_instr(0x5450);
    assert_eq!(chip8.regs.pc, 0x212);
}

#[test]
fn chip8_add_byte() {
    let mut chip8 = Chip8::new();
    chip8.exec_instr(0x70FF);
    assert_eq!(chip8.regs.v[0], 0xFF);
    chip8.exec_instr(0x7020);
    assert_eq!(chip8.regs.v[0], 0x1F);
    chip8.exec_instr(0x7A25);
    assert_eq!(chip8.regs.v[10], 0x25);
}
