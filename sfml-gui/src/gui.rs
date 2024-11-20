use crate::control::*;

use gbrs_core::cpu::Cpu;
use gbrs_core::constants::*;

use sfml::graphics::*;
use sfml::window::*;
use sfml::SfBox;
use sfml::system::*;
use sfml::audio::{Sound, SoundBuffer, SoundStatus, SoundSource};

pub const STEP_BY_STEP: bool = false;
// NOTE: This debug option is only supported on macOS. See note below
pub const DRAW_FPS: bool = false;

pub static mut SOUND_BACKING_STORE: [i16; SOUND_BUFFER_SIZE] = [0; SOUND_BUFFER_SIZE];
pub static mut SOUND_BUFFER: Option<SfBox<SoundBuffer>> = None;
pub static mut SOUND: Option<Sound> = None;

pub fn run_gui (mut gameboy: Cpu) {
    let sw = SCREEN_WIDTH as u32; let sh = SCREEN_HEIGHT as u32;
    let window_width: u32 = 640;
    let window_height: u32 = 512;

    let style = Style::RESIZE | Style::TITLEBAR | Style::CLOSE;
    let mut window = RenderWindow::new(
        (window_width, window_height),
        &format!("{} - gbrs (SFML)", gameboy.cart_info.title)[..],
        style,
        &Default::default()
    );
    // window.set_framerate_limit(gameboy.frame_rate as u32);

    let mut screen_texture = Texture::new().unwrap();
    if !screen_texture.create(sw, sh) {
        panic!("Failed to create screen texture");
    }
    // Scale the 160x144 image to the appropriate resolution
    let sprite_scale = Vector2f::new(
        window_width as f32 / sw as f32,
        window_height as f32 / sh as f32
    );

    let mut clock = Clock::start();

    let font;
    let mut text = None;
    if DRAW_FPS {
        // NOTE: DRAW_FPS only works on macOS at the moment due to hardcoded
        //   font paths. I don't want to include a font in the gbrs repo just
        //   for this debug feature.
        font = Font::from_file("/System/Library/Fonts/Menlo.ttc").unwrap();
        text = Some(Text::new("", &font, 32));
        // Make it stick out instead of white on a black+white screen
        text.as_mut().unwrap().set_fill_color(Color::BLUE);
    }

    // Get the initial frame & buffer of audio
    gameboy.step_until_full_audio_buffer();

    loop {
        let secs = clock.restart().as_seconds();

        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => {
                    window.close();
                    return;
                },
                _ => {}
            }
        }

        update_joypad_state(&mut gameboy);
        // gameboy.step_until_full_audio_buffer();

        // Draw the previous frame
        unsafe {
            screen_texture.update_from_pixels(&gameboy.gpu.get_rgba_frame(), sw, sh, 0, 0);
        }
        let mut screen_sprite = Sprite::with_texture(&screen_texture);
        screen_sprite.set_scale(sprite_scale);

        window.clear(Color::BLACK);
        window.draw(&screen_sprite);
        if DRAW_FPS {
            text.as_mut().unwrap().set_string(&format!("{} FPS", (1. / secs) as usize)[..]);
            window.draw(text.as_ref().unwrap());
        }
        window.display();

        // Play the audio while creating the next frame and sound buffer
        // This way we're not idling, we're actively computing the next event.
        // let sound_buffer = SoundBuffer::from_samples(&gameboy.mem.apu.buffer, 2, SOUND_SAMPLE_RATE as u32).unwrap();
        // let mut sound = Sound::with_buffer(&sound_buffer);
        // sound.play();

        unsafe {
            SOUND_BACKING_STORE = gameboy.mem.apu.buffer.clone();
            SOUND_BUFFER = Some(SoundBuffer::from_samples(&SOUND_BACKING_STORE, 2, SOUND_SAMPLE_RATE as u32).unwrap());
            SOUND = Some(Sound::with_buffer(match &SOUND_BUFFER {
                Some(buff) => buff,
                None => unreachable!()
            }));
            match &mut SOUND {
                Some(sound) => {
                    // sound.set_volume(0.);
                    sound.play();
                    while sound.status() == SoundStatus::PLAYING {
                        if !gameboy.mem.apu.buffer_full {
                            gameboy.step();
                        } else {
                            // We're finished with this frame. Let's just wait for audio
                            // to sync up.
                            std::hint::spin_loop();
                        }
                    }
                },
                None => unreachable!()
            }
        }

        // Just in-case we're running too slow, let's catch up.
        // This may be when you get a small audio pop. It happens more often
        // on slower machines.
        while !gameboy.mem.apu.buffer_full {
            gameboy.step();
        }
        gameboy.mem.apu.buffer_full = false;
    }
}
