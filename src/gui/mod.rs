use std::time::{Instant, Duration};
use std::thread;

use crate::gameboy::cpu::Cpu;
use crate::gameboy::constants::*;

use sfml::graphics::*;
use sfml::window::*;
use sfml::system::*;
// TODO: Audio

pub fn run_gui (mut gameboy: Cpu) {
    let sw = SCREEN_WIDTH as u32; let sh = SCREEN_HEIGHT as u32;
    let window_width: u32 = 1280;
    let window_height: u32 = 1024;

    let style = Style::RESIZE | Style::TITLEBAR | Style::CLOSE;
    let mut window = RenderWindow::new(
        (window_width, window_height),
        &format!("{} - gbrs", gameboy.mem.rom.cart_info.title)[..],
        style,
        &Default::default()
    );

    let mut screen_texture = Texture::new(sw, sh).unwrap();
    // Scale the 160x144 image to the appropriate resolution
    let sprite_scale = Vector2f::new(
        window_width as f32 / sw as f32,
        window_height as f32 / sh as f32
    );

    // 1000000000 is 1 second in nanoseconds
    let frame_time = Duration::new(0, 1000000000 / FRAME_RATE as u32);
    loop {
        let start_time = Instant::now();

        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => {
                    window.close();
                    return;
                },
                _ => {}
            }
        }

        let mut cycles = 0;
        while cycles < CYCLES_PER_FRAME {
            cycles += gameboy.step();
        }

        unsafe {
            screen_texture.update_from_pixels(&gameboy.gpu.get_sfml_frame(), sw, sh, 0, 0);
        }
        let mut screen_sprite = Sprite::with_texture(&screen_texture);
        screen_sprite.set_scale(sprite_scale);

        window.clear(Color::BLACK);
        window.draw(&screen_sprite);
        window.display();

        // Ensure we run at 60FPS
        let elapsed = start_time.elapsed();
        if elapsed < frame_time {
            thread::sleep(frame_time - elapsed);
        } else {
            println!("Running slower than 60FPS");
        }
    }
}
