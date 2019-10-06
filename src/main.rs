extern crate libc;
extern crate sdl2;
extern crate sdl2_sys;

use std::ffi::{CStr, CString};
use std::time::Duration;

use rand::random;
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
        sdl2_sys::SDL_SetHint(CString::new("SDL_RENDER_SCALE_QUALITY").unwrap().as_ptr(), CString::new("SDL_RENDER_SCALE_QUALITY").unwrap().as_ptr());
        let err = sdl2_sys::SDL_RenderSetLogicalSize(sdl2_sys::SDL_GetRenderer(canvas.window_mut().raw()), chip8::DISPLAY_WIDTH as i32, chip8::DISPLAY_HEIGHT as i32);
        //let err = sdl2_sys::SDL_RenderSetLogicalSize(sdl2_sys::SDL_GetRenderer(canvas.window_mut().raw()), chip8::DISPLAY_WIDTH as i32, chip8::DISPLAY_HEIGHT as i32);
        if err != 0 {
            panic!("ERR setting resolution {}", CStr::from_ptr(sdl2_sys::SDL_GetError()).to_str().unwrap())
        }
    }

    canvas.window_mut().set_size((chip8::DISPLAY_WIDTH * DISPLAY_SCALE) as u32, (chip8::DISPLAY_HEIGHT * DISPLAY_SCALE) as u32).unwrap();

    let mut display = [Color::RGB(0, 0, 0); chip8::DISPLAY_WIDTH * chip8::DISPLAY_HEIGHT];

    for i in 0..chip8::DISPLAY_WIDTH {
        for j in 0..chip8::DISPLAY_HEIGHT {
            display[i * chip8::DISPLAY_HEIGHT + j] = Color::RGB(random(), random(), random());
        }
    }

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for i in 0..chip8::DISPLAY_WIDTH {
            for j in 0..chip8::DISPLAY_HEIGHT {
                canvas.set_draw_color(display[i * chip8::DISPLAY_HEIGHT + j]);
                canvas.draw_point(sdl2::rect::Point::new(i as i32, j as i32)).unwrap();
            }
        }
        canvas.present();

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}