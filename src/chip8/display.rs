const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

pub struct Display {
    pub pixels: Vec<u8>,
}

impl Display {
    pub fn new() -> Display {
        Display {
            pixels: vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8],
        }
    }

    pub fn clear(&mut self) {
        for pixel in &mut self.pixels {
            *pixel = 0;
        }
    }
}

#[test]
fn display_is_blank_at_init() {
    let d = Display::new();
    for pixel in d.pixels {
        assert_eq!(pixel, 0);
    }
}

#[test]
fn clear_display() {
    let mut d = Display::new();
    d.pixels = vec![0xFF; DISPLAY_WIDTH * DISPLAY_HEIGHT / 8];
    d.clear();
    for pixel in d.pixels {
        assert_eq!(pixel, 0)
    }
}