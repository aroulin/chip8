extern crate libc;
extern crate sdl2;
extern crate sdl2_sys;

use std::ffi::CStr;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

pub mod chip8;

const DISPLAY_SCALE: usize = 8;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Chip8", chip8::DISPLAY_WIDTH as u32, chip8::DISPLAY_HEIGHT as u32)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    unsafe {
        let err = sdl2_sys::SDL_RenderSetLogicalSize(sdl2_sys::SDL_GetRenderer(canvas.window_mut().raw()), chip8::DISPLAY_WIDTH as i32, chip8::DISPLAY_HEIGHT as i32);
        //let err = sdl2_sys::SDL_RenderSetLogicalSize(sdl2_sys::SDL_GetRenderer(canvas.window_mut().raw()), chip8::DISPLAY_WIDTH as i32, chip8::DISPLAY_HEIGHT as i32);
        if err != 0 {
            panic!("ERR setting resolution {}", CStr::from_ptr(sdl2_sys::SDL_GetError()).to_str().unwrap())
        }
    }

// canvas.window_mut().set_size((chip8::DISPLAY_WIDTH * DISPLAY_SCALE) as u32, (chip8::DISPLAY_HEIGHT * DISPLAY_SCALE) as u32).unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        for i in 0..chip8::DISPLAY_WIDTH {
            canvas.set_draw_color(Color::RGB(i as u8, 0, 0));
            canvas.draw_line(sdl2::rect::Point::new(i as i32, 0), sdl2::rect::Point::new(i as i32, chip8::DISPLAY_HEIGHT as i32)).unwrap();
        }
        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}