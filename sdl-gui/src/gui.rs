use gbrs_core::cpu::Cpu;
use gbrs_core::constants::*;

use gbrs_core::lcd::GreyShade;
use sdl2::pixels::{Color, PixelFormat, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::rect::Point;
use sdl2::render::TextureAccess;
use std::time::Duration;

pub fn run_gui (mut gameboy: Cpu) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window_title = format!("gbrs - {}", gameboy.cart_info.title);
    let window = video_subsystem.window(&window_title[..], 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    // Create a texture the size of the gameboy screen
    // access: Steaming means we intend to update the texture's byte-array with
    // a full new frame each loop.
    let mut texture = texture_creator.create_texture(
        Some(PixelFormatEnum::ARGB8888), TextureAccess::Streaming, 160, 144)
        .unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        // canvas.set_draw_color(Color::RGB(255, 255, 255));
        // canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                _ => {}
            }
        }

        gameboy.mem.joypad.start_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Return);
        gameboy.mem.joypad.select_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Backspace);
        gameboy.mem.joypad.a_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::X);
        gameboy.mem.joypad.b_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Z);
        gameboy.mem.joypad.left_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Left);
        gameboy.mem.joypad.right_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Right);
        gameboy.mem.joypad.up_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Up);
        gameboy.mem.joypad.down_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Down);
        gameboy.step_one_frame();

        // canvas.with_texture_canvas(&mut texture, |texture_canvas| {
        //     for x in 0..160 {
        //         for y in 0..144 {
        //             let i = y * 160 + x;
        //             match &gameboy.gpu.finished_frame[i] {
        //                 GreyShade::White => {
        //                     texture_canvas
        //                         .set_draw_color(Color::RGB(0xDD, 0xDD, 0xDD));
        //                 },
        //                 GreyShade::LightGrey => {
        //                     texture_canvas
        //                         .set_draw_color(Color::RGB(0xAA, 0xAA, 0xAA));
        //                 },
        //                 GreyShade::DarkGrey => {
        //                     texture_canvas
        //                         .set_draw_color(Color::RGB(0x88, 0x88, 0x88));
        //                 },
        //                 GreyShade::Black => {
        //                     texture_canvas
        //                         .set_draw_color(Color::RGB(0x55, 0x55, 0x55));
        //                 }
        //             }
        //             texture_canvas
        //                 .draw_point(Point::new(x as i32, y as i32))
        //                 .unwrap();
        //         }
        //     }
        // }).unwrap();

        texture.with_lock(None, |frame, _pitch| {
            for i in 0..SCREEN_BUFFER_SIZE {
                let start = i * 4;
                match &gameboy.gpu.finished_frame[i] {
                    GreyShade::White => {
                        frame[start] = 0xDD;
                        frame[start + 1] = 0xDD;
                        frame[start + 2] = 0xDD;
                        frame[start + 3] = 0xFF;
                    },
                    GreyShade::LightGrey => {
                        frame[start] = 0xAA;
                        frame[start + 1] = 0xAA;
                        frame[start + 2] = 0xAA;
                        frame[start + 3] = 0xFF;
                    },
                    GreyShade::DarkGrey => {
                        frame[start] = 0x88;
                        frame[start + 1] = 0x88;
                        frame[start + 2] = 0x88;
                        frame[start + 3] = 0xFF;
                    },
                    GreyShade::Black => {
                        frame[start] = 0x55;
                        frame[start + 1] = 0x55;
                        frame[start + 2] = 0x55;
                        frame[start + 3] = 0xFF;
                    }
                }
            }
        }).unwrap();

        canvas.copy(&texture, None, None)
            .unwrap();

        canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
    }
}
