mod registers;
mod display;

use registers::Registers;
use display::Display;

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

    //TODO: write nibble extractor to match
    fn exec_instr(&mut self, instr: u8) {
        match instr {
            0x00E0 => self.display.clear(),
            _ => panic!("Unknown instruction {:X}", instr)
        }
    }
}
