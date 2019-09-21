use display::Display;
use registers::Registers;

use crate::utils::*;

mod registers;
mod display;

#[cfg(test)]
mod chip8_tests;

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

        // pc now points to next instruction
        self.regs.pc += 2;

        match instr_fields {

            // 00E0 - CLS - Clear the display
            (0x0, 0x0, 0xE, 0x0) =>
                self.display.clear(),

            // 00EE - RET - Return from a subroutine
            (0x0, 0x0, 0xE, 0xE) => {
                if self.regs.sp == 0 {
                    panic!("Stack underflow at instruction {:X}", self.regs.pc - 2)
                }
                self.regs.sp -= 1;
                self.regs.pc = self.regs.stack[self.regs.sp];
                self.regs.stack[self.regs.sp + 1] = 0; // clear stack
            }

            // 1nnn - JP addr - Jump to location nnn
            (0x1, n1, n2, n3) =>
                self.regs.pc = make_tribble(n1, n2, n3),

            // 2nnn - CALL addr - Call subroutine at nnn
            (0x2, n1, n2, n3) => {
                self.regs.sp += 1;
                if self.regs.sp >= self.regs.stack.len() {
                    panic!("Stack overflow at instruction {:X}", self.regs.pc - 2)
                }
                self.regs.stack[self.regs.sp - 1] = self.regs.pc;
                self.regs.pc = make_tribble(n1, n2, n3);
            }

            // 3xkk - SE Vx, byte - Skip next instruction if Vx = kk
            (0x3, x, k1, k2) =>
                if self.regs.v[x as usize] == make_byte(k1, k2) {
                    self.regs.pc += 2;
                }

            // 4xkk - SNE Vx, byte - Skip next instruction if Vx != kk
            (0x4, x, k1, k2) =>
                if self.regs.v[x as usize] != make_byte(k1, k2) {
                    self.regs.pc += 2;
                }

            // 5xy0 - SE Vx, Vy - Skip next instruction if Vx = Vy
            (0x5, x, y, 0) =>
                if self.regs.v[x as usize] == self.regs.v[y as usize] {
                    self.regs.pc += 2;
                }

            // 6xkk - LD Vx, byte - Set Vx = kk
            (0x6, x, k1, k2) =>
                self.regs.v[x as usize] = make_byte(k1, k2),

            (0x8, x, y, op) => {
                let operation: fn(u8, u8) -> u8 = match op {
                    0 => | _ , y | y,       // 8xy0 - LD Vx, Vy  - Set Vx = Vy
                    1 => | x, y| (x | y),   // 8xy1 - OR Vx, Vy  - Set Vx = Vx OR Vy
                    2 => | x, y| (x & y),   // 8xy2 - AND Vx, Vy - Set Vx = Vx AND Vy
                    3 => | x, y| (x ^ y),   // 8xy3 - XOR Vx, Vy - Set Vx = Vx XOR Vy
                    _ => panic!("Unknown opcode in instruction {:X}", instr)
                };
                self.regs.v[x as usize] = operation(self.regs.v[x as usize],
                                                    self.regs.v[y as usize])
            }

            // 7xkk - ADD Vx, byte - Set Vx = Vx + kk
            (0x7, x, k1, k2) => {
                let vx = &mut self.regs.v[x as usize];
                *vx = vx.wrapping_add(make_byte(k1, k2))
            }
            _ =>
                panic!("Unknown instruction {:X}", instr)
        } // end match instr
    } // end exec_instr
} // end impl Chip8
