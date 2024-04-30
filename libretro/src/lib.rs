use gbrs_core::constants::*;
use gbrs_core::cpu::Cpu;
use gbrs_core::config::Config;
use gbrs_core::memory::rom::Rom;
use libretro_rs::ffi::retro_log_level::*;
use libretro_rs::c_utf8::{c_utf8, CUtf8};
use libretro_rs::retro::env::{Init, UnloadGame};
use libretro_rs::retro::pixel::{Format, XRGB8888};
use libretro_rs::retro::*;
use libretro_rs::{ext, libretro_core};

struct LibretroCore {
    gameboy: Cpu,
    last_cpu_config: Config,
    rendering_mode: SoftwareRenderEnabled,
    frame_buffer: [XRGB8888; SCREEN_WIDTH * SCREEN_HEIGHT],
    pixel_format: Format<XRGB8888>,
}

static mut LOGGER: Option<PlatformLogger> = None;

impl<'a> Core<'a> for LibretroCore {
    type Init = ();

    fn get_system_info() -> SystemInfo {
        SystemInfo::new(
            c_utf8!("gbrs"),
            c_utf8!(env!("CARGO_PKG_VERSION")),
            ext!["gb","gbc"]
        )
    }

    fn init(env: &mut impl Init) -> Self::Init {
        unsafe {
            LOGGER = Some(env.get_log_interface().unwrap());
        }
        unsafe {
            gbrs_core::callbacks::set_callbacks(gbrs_core::callbacks::Callbacks {
                log: |log_str| {
                    let null_terminated = &format!("{}\0",log_str)[..];
                    let retro_str = CUtf8::from_str(null_terminated).unwrap();
                    LOGGER.unwrap().log(RETRO_LOG_INFO, retro_str)
                },
                save: |_game_name, _rom_path, _save_data| {},
                load: |_game_name, _rom_path, expected_size| vec![0; expected_size]
            })
        }
    }

    fn get_system_av_info(&self, _env: &mut impl env::GetAvInfo) -> SystemAVInfo {
        SystemAVInfo::new(
            GameGeometry::fixed(SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16),
            SystemTiming::new(DEFAULT_FRAME_RATE as f64, SOUND_SAMPLE_RATE as f64)
        )
    }

    fn run(&mut self, _env: &mut impl env::Run, runtime: &mut impl Callbacks) -> InputsPolled {
        let gb = &mut self.gameboy;

        for i in 0..SCREEN_BUFFER_SIZE {
            let mut pixel: u32 = 0;
            let r = gb.gpu.finished_frame[i].red;
            let g = gb.gpu.finished_frame[i].green;
            let b = gb.gpu.finished_frame[i].blue;
            pixel |= b as u32;
            pixel |= (g as u32) << 8;
            pixel |= (r as u32) << 16;
            self.frame_buffer[i] = XRGB8888::new_with_raw_value(pixel);
        }

        let frame = Frame::new(&self.frame_buffer, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
        runtime.upload_video_frame(&self.rendering_mode, &self.pixel_format, &frame);
        runtime.upload_audio_frame(&gb.mem.apu.buffer);

        let inputs_polled = runtime.poll_inputs();
        let port = DevicePort::new(0);
        gb.mem.joypad.a_pressed = runtime.is_joypad_button_pressed(port, JoypadButton::A);
        gb.mem.joypad.b_pressed = runtime.is_joypad_button_pressed(port, JoypadButton::B);
        gb.mem.joypad.start_pressed = runtime.is_joypad_button_pressed(port, JoypadButton::Start);
        gb.mem.joypad.select_pressed = runtime.is_joypad_button_pressed(port, JoypadButton::Select);
        gb.mem.joypad.left_pressed = runtime.is_joypad_button_pressed(port, JoypadButton::Left);
        gb.mem.joypad.right_pressed = runtime.is_joypad_button_pressed(port, JoypadButton::Right);
        gb.mem.joypad.up_pressed = runtime.is_joypad_button_pressed(port, JoypadButton::Up);
        gb.mem.joypad.down_pressed = runtime.is_joypad_button_pressed(port, JoypadButton::Down);

        while !gb.mem.apu.buffer_full {
            gb.step();
        }
        gb.mem.apu.buffer_full = false;

        inputs_polled
    }

    fn load_game<E: env::LoadGame>(
      game: &GameInfo,
      args: LoadGameExtraArgs<'a, '_, E, Self::Init>,
    ) -> Result<Self, CoreError> {
      let LoadGameExtraArgs {
        env,
        pixel_format,
        rendering_mode,
        ..
      } = args;
      let pixel_format = env.set_pixel_format_xrgb8888(pixel_format)?;
      let data: &[u8] = game.as_data().ok_or(CoreError::new())?.data();
      let config = Config {
          sound_buffer_size: SOUND_BUFFER_SIZE,
          sound_sample_rate: SOUND_SAMPLE_RATE,
          rom: Rom::from_bytes(data.to_vec())
      };
      Ok(Self {
        rendering_mode,
        pixel_format,
        gameboy: Cpu::from_config(config.clone()),
        last_cpu_config: config,
        frame_buffer: [XRGB8888::DEFAULT; SCREEN_WIDTH * SCREEN_HEIGHT],
      })
    }

    fn reset(&mut self, _env: &mut impl env::Reset) {
      self.gameboy = Cpu::from_config(self.last_cpu_config.clone());
    }

    fn unload_game(self, _env: &mut impl UnloadGame) -> Self::Init {
      ()
    }
}

libretro_core!(crate::LibretroCore);
