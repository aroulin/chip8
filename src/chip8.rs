use std::thread::sleep;
use std::time::{Duration, Instant};

use display::Display;
use display::Sprite;
use registers::Registers;

use crate::chip8::display::FONT;

mod registers;
mod display;

#[cfg(test)]
mod chip8_tests;

const MEM_SIZE: usize = 4 * 1024;
const KBD_SIZE: usize = 16;

pub struct Chip8 {
    running: bool,
    memory: Vec<u8>,
    regs: Registers,
    display: Display,
    keyboard: Vec<bool>,

    /// legacy mode:
    /// SHR Vx, Vy => VF = Vy & 1; Vx = Vy >> 1;
    /// SHL Vx, Vy => VF = Vy & 1; Vx = Vy << 1;
    ///
    /// Non-legacy-mode:
    /// SHR Vx, Vy => VF = Vx & 1; Vx = Vx >> 1
    /// SHL Vx, Vy => VF = Vx & 1; Vx = Vx << 1;
    legacy_mode: bool,

    render: Option<fn(&Vec<Vec<u8>>)>,
    play_sound: Option<fn()>,
}

const INSTR_SIZE: u16 = 2;

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
            _ => panic!("Unknown instruction opcode {:X} when decoding", op)
        }
    }
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            running: false,
            memory: vec![0; MEM_SIZE],
            regs: Registers::new(),
            display: Display::new(),
            keyboard: vec![true; KBD_SIZE],
            legacy_mode: false,
            render: None,
            play_sound: None,
        };

        // store font data
        for i in 0..FONT.len() {
            chip8.memory[i] = FONT[i];
        }

        chip8
    }

    pub fn new_with_backend(render: fn(&Vec<Vec<u8>>), play_sound: fn()) -> Chip8 {
        let mut chip8 = Chip8::new();
        chip8.render = Some(render);
        chip8.play_sound = Some(play_sound);
        chip8
    }

    pub fn run(&mut self) -> Result<(), std::io::Error> {
        self.running = true;
        while self.running {
            let old_frame_time = Instant::now();

            while old_frame_time.elapsed() < Duration::from_millis(1000 / 60 /* 1/60Hz */) {
                let pc = self.regs.pc as usize;
                let instr = ((self.memory[pc] as u16) << 8) | (self.memory[pc + 1] as u16);
                self.exec_instr(instr);
                sleep(Duration::from_millis(1000 / 500 /* 1 / 500Hz */));
            }

            if self.regs.st > 0 {
                self.regs.st -= 1;
            }

            if self.regs.dt > 0 {
                if let Some(play_sound) = self.play_sound {
                    play_sound();
                }
                self.regs.dt -= 1;
            }

            if let Some(render) = self.render {
                render(self.pixels());
            }
        } // end while(running)

        return Ok(());
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn pixels(&self) -> &Vec<Vec<u8>> {
        self.display.pixels()
    }

    fn exec_instr(&mut self, instr: u16) {
        // pc now points to next instruction
        self.regs.pc += INSTR_SIZE;

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
                    self.regs.pc += INSTR_SIZE;
                }

            // 4xkk - SNE Vx, byte - Skip next instruction if Vx != kk
            Opcode::RegImm { op: 4, x, kk } =>
                if self.regs.v[x] != kk {
                    self.regs.pc += INSTR_SIZE;
                }

            // 5xy0 - SE Vx, Vy - Skip next instruction if Vx = Vy
            Opcode::RegReg { op: 5, x, y, op2: 0 } =>
                if self.regs.v[x] == self.regs.v[y] {
                    self.regs.pc += INSTR_SIZE;
                }

            // 6xkk - LD Vx, byte - Set Vx = kk
            Opcode::RegImm { op: 6, x, kk } => self.regs.v[x] = kk,

            // 7xkk - ADD Vx, byte - Set Vx = Vx + kk
            Opcode::RegImm { op: 7, x, kk } => self.regs.v[x] = self.regs.v[x].wrapping_add(kk),

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
                let (res, carry) = self.regs.v[x].overflowing_add(self.regs.v[y]);
                self.regs.v[x] = res;
                self.regs.v[0xF] = carry as u8;
            }

            // 8xy5 - SUB Vx, Vy - Set Vx = Vx - Vy, set VF = NOT borrow
            Opcode::RegReg { op: 8, x, y, op2: 5 } => {
                let (res, borrow) = self.regs.v[x].overflowing_sub(self.regs.v[y]);
                self.regs.v[x] = res;
                self.regs.v[0xF] = (!borrow) as u8;
            }

            // 8xy6 - SHR Vx {, Vy} - Set Vx = Vx SHR 1
            Opcode::RegReg { op: 8, x, y, op2: 6 } => {
                if self.legacy_mode {
                    self.regs.v[0xF] = self.regs.v[y] & 1;
                    self.regs.v[x] = self.regs.v[y] >> 1;
                } else {
                    self.regs.v[0xF] = self.regs.v[x] & 1;
                    self.regs.v[x] >>= 1;
                }
            }

            // 8xy7 - SUBN Vx, Vy - Set Vx = Vy - Vx, set VF = NOT borrow
            Opcode::RegReg { op: 8, x, y, op2: 7 } => {
                let (res, borrow) = self.regs.v[y].overflowing_sub(self.regs.v[x]);
                self.regs.v[x] = res;
                self.regs.v[0xF] = (!borrow) as u8;
            }

            // 8xyE - SHL Vx {, Vy} - Set Vx = Vx SHL 1
            Opcode::RegReg { op: 8, x, y, op2: 0xE } => {
                if self.legacy_mode {
                    self.regs.v[0xF] = (self.regs.v[y] >> 7) & 1;
                    self.regs.v[x] = self.regs.v[y] << 1;
                } else {
                    self.regs.v[0xF] = (self.regs.v[x] >> 7) & 1;
                    self.regs.v[x] <<= 1;
                }
            }

            // 9xy0 - SNE Vx, Vy - Skip next instruction if Vx != Vy
            Opcode::RegReg { op: 9, x, y, op2: 0 } => {
                if self.regs.v[x] == self.regs.v[y] {
                    self.regs.pc += INSTR_SIZE;
                }
            }

            // Annn - LD I, addr - Set I = nnn
            Opcode::Imm { op: 0xA, nnn } => self.regs.i = nnn as usize,

            // Bnnn - JP V0, addr - Jump to location nnn + V0 //TODO: overflow?
            Opcode::Imm { op: 0xB, nnn } => self.regs.pc = nnn + self.regs.v[0] as u16,

            // Cxkk - RND Vx, byte - Set Vx = random byte AND kk
            Opcode::RegImm { op: 0xC, x, kk } => self.regs.v[x] = rand::random::<u8>() & kk,

            // Dxyn - DRW Vx, Vy, nibble
            // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            Opcode::RegReg { op: 0xD, x, y, op2: n } => {
                let mut sprite_bytes = Vec::new();
                for i in 0..n {
                    let byte = self.memory[self.regs.i + i as usize];
                    sprite_bytes.push(byte);
                }

                let sprite = Sprite::new(sprite_bytes);
                let collision = self.display.draw_sprite(&sprite, x, y);
                self.regs.v[0xF] = collision as u8;
            }

            // Ex9E - SKP Vx - Skip next instruction if key with the value of Vx is pressed
            // ExA1 - SKNP Vx - Skip next instruction if key with the value of Vx is not pressed
            Opcode::RegImm { op: 0xE, x, kk } if kk == 0x9E || kk == 0xA1 => {
                let key = self.regs.v[x] as usize;
                if key > 0xF {
                    panic!("instruction {:X} executed with Vx ({:X}) > 0xF", instr, key);
                }

                let pressed = self.keyboard[key];

                if (kk == 0x9E && pressed) || (kk == 0xA1 && !pressed) {
                    self.regs.pc += 2;
                }
            }

            // Fx07 - LD Vx, DT - Set Vx = delay timer value
            Opcode::RegImm { op: 0xF, x, kk: 0x07 } => self.regs.v[x] = self.regs.dt,

            // Fx0A - LD Vx, K - Wait for a key press, store the value of the key in Vx
            Opcode::RegImm { op: 0xF, x, kk: 0x0A } => {
                //TODO: optimize waiting
                let mut key_pressed = None;
                while key_pressed == None {
                    key_pressed = self.keyboard.iter().position(|k| *k);
                };

                self.regs.v[x] = key_pressed.unwrap() as u8;
            }

            // Fx15 - LD DT, Vx - Set delay timer = Vx
            Opcode::RegImm { op: 0xF, x, kk: 0x15 } => self.regs.dt = self.regs.v[x],

            // Fx18 - LD ST, Vx - Set sound timer = Vx
            Opcode::RegImm { op: 0xF, x, kk: 0x18 } => self.regs.st = self.regs.v[x],

            // Fx1E - ADD I, Vx => Set I = I + Vx
            Opcode::RegImm { op: 0xF, x, kk: 0x1E } =>
                self.regs.i = self.regs.i.wrapping_add(self.regs.v[x] as usize),

            // Fx29 - LD F, Vx - Set I = location of sprite for digit Vx
            Opcode::RegImm { op: 0xF, x, kk: 0x29 } => {
                let value = self.regs.v[x];
                if value > 0xF {
                    panic!("instr {:X}: Vx {:X} must be a digit not larger than 0xF", instr, value)
                }
                self.regs.i = (value as usize) * 5; /* five bytes per font digit */
            }

            // Fx33 - LD B, Vx - Store BCD representation of Vx in memory locations I, I+1, and I+2
            Opcode::RegImm { op: 0xF, x, kk: 0x33 } => {
                let value = self.regs.v[x];
                self.memory[self.regs.i + 2] = value % 10;
                self.memory[self.regs.i + 1] = (value / 10) % 10;
                self.memory[self.regs.i] = (value / 100) % 10;
            }

            // Fx55 - LD [I], Vx - Store registers V0 through Vx in memory starting at location I
            Opcode::RegImm { op: 0xF, x, kk: 0x55 } => {
                for i in 0..(x + 1) {
                    self.memory[self.regs.i + i] = self.regs.v[i];
                }

                if self.legacy_mode {
                    self.regs.i += x + 1;
                }
            }

            // Fx65 - LD Vx, [I] - Read registers V0 through Vx from memory starting at location I
            Opcode::RegImm { op: 0xF, x, kk: 0x65 } => {
                for i in 0..(x + 1) {
                    self.regs.v[i] = self.memory[self.regs.i + i];
                }

                if self.legacy_mode {
                    self.regs.i += x + 1;
                }
            }

            _ =>
                panic!("Unknown instruction {:X}", instr)
        } // end match instr
    } // end exec_instr
} // end impl Chip8
