use chip8::Chip8;

pub mod chip8;

fn main() {
    let mut chip8 = Chip8::new();
    let _pixels = chip8.pixels();
    chip8.run().unwrap();

    println!("Hello, world!");
}