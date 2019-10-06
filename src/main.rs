extern crate libc;
extern crate sdl2;
extern crate sdl2_sys;

use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::prelude::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use crate::chip8::Chip8;

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
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut check_input = |running: &mut bool, _keyboard: &mut Vec<bool>| {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    *running = false;
                }
                _ => {}
            }
        }
    };

    let mut render = |display: Vec<Vec<u8>>| {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for i in 0..chip8::DISPLAY_HEIGHT {
            for j in 0..chip8::DISPLAY_WIDTH {
                let color = if display[i][j] == 1 {
                    Color::RGB(0, 0, 0)
                } else {
                    Color::RGB(255, 255, 255)
                };
                canvas.set_draw_color(color);
                canvas.draw_point(sdl2::rect::Point::new(j as i32, i as i32)).unwrap();
            }
        }
        canvas.present();
    };

    let play_sound = || {};

    let mut chip8 = Chip8::new_with_backend(&mut render, &play_sound, &mut check_input);
    let mut f = File::open("test.ch8").unwrap();
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer).unwrap();
    chip8.load_rom(buffer).unwrap();
    chip8.run().unwrap();
}