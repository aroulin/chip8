pub struct Registers {
    pub v: Vec<u8>,
    pub i: u16,
    pub pc: u16,
    pub sp: u8,
    pub stack: Vec<u16>,
    /// delay timer
    pub dt: u8,
    /// sound timer
    pub st: u8,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            v: vec![0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: vec![0; 16],
            dt: 0,
            st: 0,
        }
    }
}