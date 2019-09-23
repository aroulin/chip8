use display::Display;
use registers::Registers;

mod registers;
mod display;

#[cfg(test)]
mod chip8_tests;

const MEM_SIZE: usize = 4 * 1024;

pub struct Chip8 {
    memory: Vec<u8>,
    regs: Registers,
    display: Display,

    /// legacy mode:
    /// SHR Vx, Vy => VF = Vy & 1; Vx = Vy >> 1;
    /// SHL Vx, Vy => VF = Vy & 1; Vx = Vy << 1;
    /// Non-legacy-mode:
    /// SHR Vx, Vy => VF = Vx & 1; Vx = Vx >> 1
    /// SHL Vx, Vy => VF = Vx & 1; Vx = Vx << 1;
    legacy_mode: bool,
}

#[derive(Debug)]
enum Opcode {
    Imm { op: u8, nnn: u16 },
    RegImm { op: u8, x: usize, kk: u8 },
    RegReg { op: u8, x: usize, y: usize, op2: u8 },
}

impl From<u16> for Opcode {
    fn from(opcode: u16) -> Self {
        let op = (opcode >> 12) as u8;
        let nnn = opcode & 0xFFF;
        let x = ((opcode >> 8) & 0xF) as usize;
        let kk = (opcode & 0xFF) as u8;
        let y = ((opcode >> 4) & 0xF) as usize;
        let op2 = (opcode & 0xF) as u8;

        match op {
            0 | 1 | 2 | 0xA | 0xB => Opcode::Imm { op, nnn },
            3 | 4 | 6 | 7 | 0xC | 0xE | 0xF => Opcode::RegImm { op, x, kk },
            5 | 8 | 9 | 0xD => Opcode::RegReg { op, x, y, op2 },
            _ => panic!("Unknown")
        }
    }
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            memory: vec![0; MEM_SIZE],
            regs: Registers::new(),
            display: Display::new(),
            legacy_mode: false,
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
        // pc now points to next instruction
        self.regs.pc += 2;

        match Opcode::from(instr) {
            // 00E0 - CLS - Clear the display
            Opcode::Imm { op: 0, nnn: 0xE0 } => self.display.clear(),

            // 00EE - RET - Return from a subroutine
            Opcode::Imm { op: 0, nnn: 0xEE } => {
                if self.regs.sp == 0 {
                    panic!("Stack underflow at instruction {:X}", self.regs.pc - 2)
                }
                self.regs.sp -= 1;
                self.regs.pc = self.regs.stack[self.regs.sp];
                self.regs.stack[self.regs.sp + 1] = 0; // clear stack
            }

            // 1nnn - JP addr - Jump to location nnn
            Opcode::Imm { op: 1, nnn } => self.regs.pc = nnn,

            // 2nnn - CALL addr - Call subroutine at nnn
            Opcode::Imm { op: 2, nnn } => {
                self.regs.sp += 1;
                if self.regs.sp >= self.regs.stack.len() {
                    panic!("Stack overflow at instruction {:X}", self.regs.pc - 2)
                }
                self.regs.stack[self.regs.sp - 1] = self.regs.pc;
                self.regs.pc = nnn;
            }

            // 3xkk - SE Vx, byte - Skip next instruction if Vx = kk
            Opcode::RegImm { op: 3, x, kk } =>
                if self.regs.v[x] == kk {
                    self.regs.pc += 2;
                }

            // 4xkk - SNE Vx, byte - Skip next instruction if Vx != kk
            Opcode::RegImm { op: 4, x, kk } =>
                if self.regs.v[x] != kk {
                    self.regs.pc += 2;
                }

            // 5xy0 - SE Vx, Vy - Skip next instruction if Vx = Vy
            Opcode::RegReg { op: 5, x, y, op2: 0 } =>
                if self.regs.v[x] == self.regs.v[y] {
                    self.regs.pc += 2;
                }

            // 6xkk - LD Vx, byte - Set Vx = kk
            Opcode::RegImm { op: 6, x, kk } => self.regs.v[x] = kk,

            // 7xkk - ADD Vx, byte - Set Vx = Vx + kk
            Opcode::RegImm { op: 7, x, kk } =>
                self.regs.v[x] = self.regs.v[x].wrapping_add(kk),

            // 8xy0 - LD Vx, Vy  - Set Vx = Vy
            Opcode::RegReg { op: 8, x, y, op2: 0 } => self.regs.v[x] = self.regs.v[y],
            // 8xy1 - OR Vx, Vy  - Set Vx = Vx OR Vy
            Opcode::RegReg { op: 8, x, y, op2: 1 } => self.regs.v[x] |= self.regs.v[y],
            // 8xy2 - AND Vx, Vy - Set Vx = Vx AND Vy
            Opcode::RegReg { op: 8, x, y, op2: 2 } => self.regs.v[x] &= self.regs.v[y],
            // 8xy3 - XOR Vx, Vy - Set Vx = Vx XOR Vy
            Opcode::RegReg { op: 8, x, y, op2: 3 } => self.regs.v[x] ^= self.regs.v[y],
            // 8xy4 - ADD Vx, Vy - Set Vx = Vx + Vy - set VF = carry
            Opcode::RegReg { op: 8, x, y, op2: 4 } => {
                let (res, overflow) = self.regs.v[x].overflowing_add(self.regs.v[y]);
                self.regs.v[x] = res;
                self.regs.v[0xF] = overflow as u8;
            }
            // 8xy5 - SUB Vx, Vy - Set Vx = Vx - Vy, set VF = NOT borrow
            Opcode::RegReg { op: 8, x, y, op2: 5 } => {
                let (res, overflow) = self.regs.v[x].overflowing_sub(self.regs.v[y]);
                self.regs.v[x] = res;
                self.regs.v[0xF] = (!overflow) as u8;
            }
            // 8xy6 - SHR Vx {, Vy} - Set Vx = Vx SHR 1
            Opcode::RegReg { op: 8, x, y, op2: 6 } => {
                if self.legacy_mode == true {
                    self.regs.v[0xF] = self.regs.v[y] & 1;
                    self.regs.v[x] = self.regs.v[y] >> 1;
                } else {
                    self.regs.v[0xF] = self.regs.v[x] & 1;
                    self.regs.v[x] >>= 1;
                }
            }

            // 8xy7 - SUBN Vx, Vy - Set Vx = Vy - Vx, set VF = NOT borrow
            Opcode::RegReg { op: 8, x, y, op2: 7 } => {
                let (res, overflow) = self.regs.v[y].overflowing_sub(self.regs.v[x]);
                self.regs.v[x] = res;
                self.regs.v[0xF] = (!overflow) as u8;
            }

            // 8xyE - SHL Vx {, Vy} - Set Vx = Vx SHL 1
            Opcode::RegReg { op: 8, x, y, op2: 0xE } => {
                if self.legacy_mode == true {
                    self.regs.v[0xF] = (self.regs.v[y] >> 7) & 1;
                    self.regs.v[x] = self.regs.v[y] << 1;
                } else {
                    self.regs.v[0xF] = (self.regs.v[x] >> 7) & 1;
                    self.regs.v[x] <<= 1;
                }
            }

            _ =>
                panic!("Unknown instruction {:X}", instr)
        } // end match instr
    } // end exec_instr
} // end impl Chip8
