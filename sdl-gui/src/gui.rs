use gbrs_core::cpu::Cpu;
use gbrs_core::constants::*;

use gbrs_core::lcd::GreyShade;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::rect::Rect;
use std::time::Duration;

// NOTE: The SDL port does not currently perform non-integer scaling.
//   Please choose a multiple of 160x144
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 720;

struct GameboyAudio {
    pub apu_buffer: [i16; SOUND_BUFFER_SIZE],
    pub called_back: bool
}
impl AudioCallback for GameboyAudio {
    type Channel = f32;

    fn callback (&mut self, out: &mut [f32]) {
        for i in 0..SOUND_BUFFER_SIZE {
            out[i] = self.apu_buffer[i] as f32 / 32767.0;
        }
        self.called_back = true;
    }
}

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

    let timer = sdl_context.timer().unwrap();
    
    let perf_freq = timer.performance_frequency();
    assert_eq!(perf_freq, 1000000000, 
        "TODO: Don't rely on hardcoded nanosecond accuracy");

    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(SOUND_SAMPLE_RATE as i32),
        channels: Some(2),
        samples: Some(SOUND_BUFFER_SIZE as u16)
    };

    // TODO: Change to SDL AudioQueue and sync graphics output based on sound
    let mut device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        println!("{:?}", spec);
        assert_eq!(spec.samples, SOUND_BUFFER_SIZE as u16, 
            "Audio device does not support desired buffer size");

        GameboyAudio {
            apu_buffer: [0; SOUND_BUFFER_SIZE],
            called_back: false
        }
    }).unwrap();

    // Start audio
    device.resume();

    let mut last_perf_counter = timer.performance_counter();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                _ => {}
            }
        }

        let new_perf_counter = timer.performance_counter();
        // Nanoseconds it took to draw the last frame
        // We can use this to roughly slow ourselves down and try to lock at
        // real gameboy speed.
        let frame_time = new_perf_counter - last_perf_counter;
        last_perf_counter = new_perf_counter;

        gameboy.mem.joypad.start_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Return);
        gameboy.mem.joypad.select_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Backspace);
        gameboy.mem.joypad.a_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::X);
        gameboy.mem.joypad.b_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Z);
        gameboy.mem.joypad.left_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Left);
        gameboy.mem.joypad.right_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Right);
        gameboy.mem.joypad.up_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Up);
        gameboy.mem.joypad.down_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::Down);
        gameboy.step_one_frame();
        // TODO: Try step_until_full_audio_buffer

        {
            // Acquire a lock. This lets us read and modify callback data.
            let mut lock = device.lock();
            (*lock).apu_buffer = gameboy.mem.apu.buffer.clone();
            // Lock guard is dropped here
        }

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
        
        // If we draw the previous frame quicker than 60FPS, slow down and wait
        // so we can get back down to speed.
        // (this does not compensate for if your machine is too slow to eumlate
        //  gameboy at 60FPS)
        let nanoseconds_in_one_frame = 1_000_000_000u32 / DEFAULT_FRAME_RATE as u32;
        if let Some(too_fast_by) = nanoseconds_in_one_frame.checked_sub(frame_time as u32) {
            ::std::thread::sleep(Duration::new(0, too_fast_by));
        }
    }
}
