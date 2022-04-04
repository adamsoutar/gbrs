use gbrs_core::cpu::Cpu;
use gbrs_core::constants::*;

use gbrs_core::lcd::GreyShade;
use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioQueue, self};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::rect::Rect;

// NOTE: The SDL port does not currently perform non-integer scaling.
//   Please choose a multiple of 160x144
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 720;

pub fn run_gui (mut gameboy: Cpu) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window_title = format!("gbrs - {}", gameboy.cart_info.title);
    let window = video_subsystem.window(&window_title[..], WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let square_width = WINDOW_WIDTH as usize / SCREEN_WIDTH;
    let square_height = WINDOW_HEIGHT as usize / SCREEN_HEIGHT;

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(SOUND_SAMPLE_RATE as i32),
        channels: Some(2),
        samples: Some(SOUND_BUFFER_SIZE as u16)
    };

    let audio_queue: AudioQueue<i16> = audio_subsystem
        .open_queue(None, &desired_spec)
        .unwrap();
    println!("{:?}", audio_queue.spec());

    gameboy.step_until_full_audio_buffer();
    // gameboy.mem.apu.buffer_full = true;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                _ => {}
            }
        }

        // Draw the screen
        for x in 0..SCREEN_WIDTH {
            for y in 0..SCREEN_HEIGHT {
                let i = (y * 160 + x) as usize;
                match &gameboy.gpu.finished_frame[i] {
                    GreyShade::White => {
                        canvas
                            .set_draw_color(Color::RGB(0xDD, 0xDD, 0xDD));
                    },
                    GreyShade::LightGrey => {
                        canvas
                            .set_draw_color(Color::RGB(0xAA, 0xAA, 0xAA));
                    },
                    GreyShade::DarkGrey => {
                        canvas
                            .set_draw_color(Color::RGB(0x88, 0x88, 0x88));
                    },
                    GreyShade::Black => {
                        canvas
                            .set_draw_color(Color::RGB(0x55, 0x55, 0x55));
                    }
                }
                canvas
                    .fill_rect(Rect::new((x * square_width) as i32, (y * square_height) as i32, square_width as u32, square_height as u32))
                    .unwrap();
            }
        }
        canvas.present();
        
        gameboy.mem.joypad.start_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Return);
        gameboy.mem.joypad.select_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Backspace);
        gameboy.mem.joypad.a_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::X);
        gameboy.mem.joypad.b_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Z);
        gameboy.mem.joypad.left_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Left);
        gameboy.mem.joypad.right_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Right);
        gameboy.mem.joypad.up_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Up);
        gameboy.mem.joypad.down_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Down);


        gameboy.step_until_full_audio_buffer();

        let pre = audio_queue.size();
        audio_queue.queue_audio(&gameboy.mem.apu.buffer).unwrap();
        audio_queue.resume();
        let diff = audio_queue.size() - pre;
        
        while audio_queue.size() > diff {
            // if !gameboy.mem.apu.buffer_full {
            //     gameboy.step();
            // }
            std::hint::spin_loop();
        }
    }
}
