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
        return Ok(());
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
            (0x7, x, k1, k2) => {
                let mut vx = &mut self.regs.v[x as usize];
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
