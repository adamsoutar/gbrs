use gbrs_core::constants::*;
use gbrs_core::cpu::Cpu;
use libretro_rs::*;

struct LibretroCore {
    gameboy: Option<Cpu>,
    last_rom_data: Vec<u8>
}

impl LibretroCore {

}

const SCREEN_BYTES_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 2;

impl RetroCore for LibretroCore {
    fn init(_: &RetroEnvironment) -> Self {
        LibretroCore {
            gameboy: None,
            last_rom_data: vec![]
        }
    }

    fn get_system_info() -> RetroSystemInfo {
        let info = RetroSystemInfo::new("gbrs", env!("CARGO_PKG_VERSION"));
        info.with_valid_extensions(&vec!["gb", "gbc"])
    }

    fn reset(&mut self, _: &libretro_rs::RetroEnvironment) {
        self.gameboy = Some(Cpu::from_rom_bytes(self.last_rom_data.clone()));
    }

    fn run(&mut self, environment: &libretro_rs::RetroEnvironment, runtime: &RetroRuntime) {
        let gb = self.gameboy.as_mut().unwrap();

        // Convert frame into 0RGB1555
        let mut screen_bytes = [0; SCREEN_BYTES_SIZE];
        for i in 0..SCREEN_BUFFER_SIZE {
            let r = gb.gpu.finished_frame[i].red;
            let g = gb.gpu.finished_frame[i].green;
            let b = gb.gpu.finished_frame[i].blue;
            let r5 = (r >> 3) as u16;
            let g5 = (g >> 3) as u16;
            let b5 = (b >> 3) as u16;
            let final_colour = (r5 << 10) | (g5 << 5) | b5;
            screen_bytes[i * 2] = (final_colour & 0xFF) as u8;
            screen_bytes[i * 2 + 1] = ((final_colour >> 8) & 0xFF) as u8;
        }

        runtime.upload_video_frame(&screen_bytes, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32, SCREEN_WIDTH * 2);
        runtime.upload_audio_frame(&gb.mem.apu.buffer);

        gb.mem.joypad.a_pressed = runtime.is_joypad_button_pressed(0, RetroJoypadButton::A);
        gb.mem.joypad.b_pressed = runtime.is_joypad_button_pressed(0, RetroJoypadButton::B);
        gb.mem.joypad.start_pressed = runtime.is_joypad_button_pressed(0, RetroJoypadButton::Start);
        gb.mem.joypad.select_pressed = runtime.is_joypad_button_pressed(0, RetroJoypadButton::Select);
        gb.mem.joypad.left_pressed = runtime.is_joypad_button_pressed(0, RetroJoypadButton::Left);
        gb.mem.joypad.right_pressed = runtime.is_joypad_button_pressed(0, RetroJoypadButton::Right);
        gb.mem.joypad.up_pressed = runtime.is_joypad_button_pressed(0, RetroJoypadButton::Up);
        gb.mem.joypad.down_pressed = runtime.is_joypad_button_pressed(0, RetroJoypadButton::Down);

        while !gb.mem.apu.buffer_full {
            gb.step();
        }
        gb.mem.apu.buffer_full = false;
    }

    fn load_game(&mut self, environment: &libretro_rs::RetroEnvironment, game: RetroGame<'_>) -> RetroLoadGameResult {
        match &game {
            RetroGame::None { meta: _ } => {
                panic!("Asked to load RetroGame::None")
            },
            RetroGame::Data { meta: _, data } => {
                self.last_rom_data = data.to_vec();
                self.gameboy = Some(Cpu::from_rom_bytes(data.to_vec()));
                environment.set_pixel_format(RetroPixelFormat::XRGB8888);
                RetroLoadGameResult::Success {
                    audio: RetroAudioInfo::new(SOUND_SAMPLE_RATE as f64),
                    video: RetroVideoInfo::new(
                        DEFAULT_FRAME_RATE as f64,
                        SCREEN_WIDTH as u32,
                        SCREEN_HEIGHT as u32)
                }
            },
            RetroGame::Path { meta: _, path: _ } => {
                panic!("Asked to load RetroGame::Path")
            }
        }
    }
}

libretro_core!(LibretroCore);
