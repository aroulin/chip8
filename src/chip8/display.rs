const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

pub struct Display {
    pixels: Vec<Vec<bool>>,
}

pub struct Sprite {
    pixels: Vec<u8>,
    height: usize,
}

impl Sprite {
    pub fn new(sprite: Vec<u8>) -> Sprite {
        assert!(sprite.len() > 1);
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
            pixels: vec![vec![false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.pixels {
            for pixel in row {
                *pixel = false;
            }
        }
    }

    pub fn pixels(&self) -> Vec<u8> {
        self.pixels.clone().into_iter().map(
            |row|
                row.clone()
                    .into_iter()
                    .fold(0, |value, bit| (value << 2) + bit as u8)
        ).collect()
    }

    pub fn draw_sprite(&mut self, sprite: &Sprite, x: usize, y: usize) -> bool {
        let mut collision = false;
        for row_index in 0..sprite.height {
            let row = (y + row_index) % DISPLAY_HEIGHT;
            for col_index in 0..8 {
                let col = (x + col_index) % DISPLAY_WIDTH;
                let pixel = self.pixels[row][col] as u8;
                let mut new_pixel = pixel ^ (sprite.pixels[row] & (128u8 >> col_index as u8));

                if pixel == 1 && new_pixel == 0 {
                    collision = true;
                }
            }
        }

        collision
    }
}

#[test]
fn display_is_blank_at_init() {
    let d = Display::new();
    for byte in d.pixels() {
        assert_eq!(byte, 0);
    }
}

#[test]
fn clear_display() {
    let mut d = Display::new();
    d.pixels = vec![vec![true; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    d.clear();
    for byte in d.pixels() {
        assert_eq!(byte, 0)
    }
}