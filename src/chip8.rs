use display::Display;
use registers::Registers;

mod registers;
mod display;

const MEM_SIZE: usize = 4 * 1024;

pub struct Chip8 {
    memory: Vec<u8>,
    regs: Registers,
    display: Display,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            memory: vec![0; MEM_SIZE],
            regs: Registers::new(),
            display: Display::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), std::io::Error> {
        self.exec_instr(0x00E0);
        return Ok(());
    }

    pub fn pixels(&self) -> &Vec<u8> {
        &self.display.pixels
    }

    fn exec_instr(&mut self, instr: u16) {
        let instr_fields =
            ((instr) >> 12,
             (instr >> 8) & 0xF,
             (instr >> 4) & 0xF,
             (instr) & 0xF);

        match instr_fields {
            (0x0, 0x0, 0xE, 0x0) =>
                self.display.clear(),

            (0x1, n1, n2, n3) =>
                self.regs.pc = make_12_bits(n1, n2, n3),

            (0x2, n1, n2, n3) => {
                self.regs.sp += 1;
                if self.regs.sp as usize >= self.regs.stack.len() {
                    panic!("Stack overflow at instruction {:X}", self.regs.pc)
                }
                self.regs.stack[(self.regs.sp - 1) as usize] = self.regs.pc;
                self.regs.pc = make_12_bits(n1, n2, n3);
            }

            (0x7, x, k1, k2) => {
                let vx = &mut self.regs.v[x as usize];
                *vx = vx.wrapping_add(make_byte(k1, k2))
            }
            _ =>
                panic!("Unknown instruction {:X}", instr)
        }
    }
}

// helpers
fn make_byte(high: u16, low: u16) -> u8 {
    assert!(high <= 0xFF);
    assert!(low <= 0xFF);
    ((high << 4) | low) as u8
}

fn make_12_bits(n1: u16, n2: u16, n3: u16) -> u16 {
    assert!(n1 <= 0xFF);
    assert!(n2 <= 0xFF);
    assert!(n3 <= 0xFF);
    ((n1 << 8) | (n2 << 4) | n3) as u16
}

#[test]
fn chip8_jmp_addr() {
    let mut chip8 = Chip8::new();
    chip8.exec_instr(0x1555);
    assert_eq!(chip8.regs.pc, 0x555);
}

#[test]
fn chip8_call_addr() {
    let mut chip8 = Chip8::new();
    chip8.exec_instr(0x2555);
    assert_eq!(chip8.regs.pc, 0x555);
    assert_eq!(chip8.regs.sp, 1);
    assert_eq!(chip8.regs.stack[0], 0x200);
    chip8.exec_instr(0x2777);
    assert_eq!(chip8.regs.pc, 0x777);
    assert_eq!(chip8.regs.sp, 2);
    assert_eq!(chip8.regs.stack[1], 0x555);
}

#[test]
#[should_panic]
fn chip8_call_addr_stack_overflow() {
    let mut chip8 = Chip8::new();
    chip8.regs.sp = 15;
    chip8.exec_instr(0x2555);
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
