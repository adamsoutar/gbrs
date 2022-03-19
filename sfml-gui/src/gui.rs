use gbrs_core::callbacks::{Callbacks, CALLBACKS};
use gbrs_core::callbacks::set_callbacks;
use gbrs_core::cpu::Cpu;
use gbrs_core::constants::*;

use sfml::graphics::*;
use sfml::window::*;
use sfml::system::*;
use sfml::audio::{Sound, SoundBuffer, SoundStatus};

pub const STEP_BY_STEP: bool = false;
pub const FPS_CYCLES_DEBUG: bool = false;

pub static mut SOUND_BUFFER: Option<SfBox<SoundBuffer>> = None;
pub static mut SOUND: Option<Sound> = None;

pub fn run_gui (mut gameboy: Cpu) {
    let sw = SCREEN_WIDTH as u32; let sh = SCREEN_HEIGHT as u32;
    let window_width: u32 = 1280;
    let window_height: u32 = 1024;

    let style = Style::RESIZE | Style::TITLEBAR | Style::CLOSE;
    let mut window = RenderWindow::new(
        (window_width, window_height),
        &format!("{} - gbrs", gameboy.cart_info.title)[..],
        style,
        &Default::default()
    );
    // window.set_framerate_limit(gameboy.frame_rate as u32);

    let mut screen_texture = Texture::new(sw, sh).unwrap();
    // Scale the 160x144 image to the appropriate resolution
    let sprite_scale = Vector2f::new(
        window_width as f32 / sw as f32,
        window_height as f32 / sh as f32
    );

    let mut clock = Clock::start();
    let mut step_last_frame = false;

    unsafe {
        set_callbacks(Callbacks {
            log: CALLBACKS.log,
            save: CALLBACKS.save,
            load: CALLBACKS.load,
            get_ms_timestamp: CALLBACKS.get_ms_timestamp,
            play_sound: |sound_buffer| {
                // HACK: *Horrible* unsafe crimes to make SOUND outlive its
                //   block and not get Dropped. When a Sound Drops out of a
                //   scope, it stops playing.
                // TODO: Is this leaking memory? Does Rust still call Drop
                //   when a mutable static is reassigned?
                SOUND_BUFFER = Some(SoundBuffer::from_samples(sound_buffer, 1, SOUND_SAMPLE_RATE as u32).unwrap());
                SOUND = Some(Sound::with_buffer(match &SOUND_BUFFER {
                    Some(buff) => buff,
                    None => unreachable!()
                }));
                match &mut SOUND {
                    Some(sound) => sound.play(),
                    None => unreachable!()
                }
            }
        })
    }

    loop {
        let secs = clock.restart().as_seconds();
        if FPS_CYCLES_DEBUG {
            println!("{} FPS", 1. / secs);
        }

        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => {
                    window.close();
                    return;
                },
                _ => {}
            }
        }

        // TODO: Raise the joypad interrupt
        gameboy.mem.joypad.a_pressed = Key::is_pressed(Key::X);
        gameboy.mem.joypad.b_pressed = Key::is_pressed(Key::Z);
        gameboy.mem.joypad.start_pressed = Key::is_pressed(Key::Return);
        gameboy.mem.joypad.select_pressed = Key::is_pressed(Key::BackSpace);
        gameboy.mem.joypad.up_pressed = Key::is_pressed(Key::Up);
        gameboy.mem.joypad.down_pressed = Key::is_pressed(Key::Down);
        gameboy.mem.joypad.left_pressed = Key::is_pressed(Key::Left);
        gameboy.mem.joypad.right_pressed = Key::is_pressed(Key::Right);

        if STEP_BY_STEP {
            let pressing_step = Key::is_pressed(Key::S);
            if pressing_step && !step_last_frame {
                gameboy.step();
            }
            step_last_frame = pressing_step;
        } else {
            let cycles = gameboy.step_until_full_audio_buffer();
            if FPS_CYCLES_DEBUG {
                println!("Ran {} cycles that frame", cycles);
            }
        }

        unsafe {
            screen_texture.update_from_pixels(&gameboy.gpu.get_sfml_frame(), sw, sh, 0, 0);
        }
        let mut screen_sprite = Sprite::with_texture(&screen_texture);
        screen_sprite.set_scale(sprite_scale);

        window.clear(Color::BLACK);
        window.draw(&screen_sprite);
        window.display();


        unsafe {
            if let Some(sound) = &SOUND {
                // Wait until we drain the audio buffer
                while sound.status() == SoundStatus::Playing {
                    std::hint::spin_loop()
                }
            }
        }
    }
}
