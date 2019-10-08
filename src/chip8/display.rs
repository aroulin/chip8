use crate::chip8::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub struct Display {
    pixels: Vec<Vec<u8>>,
}

pub struct Sprite {
    pixels: Vec<u8>,
    height: usize,
}

impl Sprite {
    pub fn new(sprite: Vec<u8>) -> Sprite {
        //assert!(sprite.len() > 1);
        assert!(sprite.len() < 16);

        Sprite {
            pixels: sprite.clone(),
            height: sprite.len(),
        }
    }
}

impl Display {
    pub fn new() -> Display {
        Display {
            pixels: vec![vec![0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.pixels {
            for pixel in row {
                *pixel = 0;
            }
        }
    }

    pub fn pixels(&self) -> &Vec<Vec<u8>> {
        &self.pixels
    }

    pub fn draw_sprite(&mut self, sprite: &Sprite, x: usize, y: usize) -> bool {
        let mut collision = false;
        for row_index in 0..sprite.height {
            let row = (y + row_index) % DISPLAY_HEIGHT;
            for col_index in 0..8 {
                let col = (x + col_index) % DISPLAY_WIDTH;
                let pixel = self.pixels[row][col];
                let new_pixel = pixel ^ ((sprite.pixels[row_index] & (128u8 >> (col_index as u8))) >> (7 - col_index) as u8);

                if pixel == 1 && new_pixel == 0 {
                    collision = true;
                }

                self.pixels[row][col] = new_pixel;
            }
        }

        collision
    }
}

#[test]
fn display_is_blank_at_init() {
    let d = Display::new();
    for byte in d.pixels().into_iter().flatten() {
        assert_eq!(*byte, 0);
    }
}

#[test]
fn clear_display() {
    let mut d = Display::new();
    d.pixels = vec![vec![0; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    d.clear();
    for byte in d.pixels().into_iter().flatten() {
        assert_eq!(*byte, 0)
    }
}

pub const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80,
];
