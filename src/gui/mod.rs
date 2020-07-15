use crate::gameboy::cpu::Cpu;
use crate::gameboy::constants::*;

use sfml::graphics::*;
use sfml::window::*;
use sfml::system::*;
// TODO: Audio

pub const FPS_CYCLES_DEBUG: bool = false;

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
    window.set_framerate_limit(60);

    let mut screen_texture = Texture::new(sw, sh).unwrap();
    // Scale the 160x144 image to the appropriate resolution
    let sprite_scale = Vector2f::new(
        window_width as f32 / sw as f32,
        window_height as f32 / sh as f32
    );

    let mut clock = Clock::start();

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
        gameboy.mem.joypad.a_pressed = Key::is_pressed(Key::Z);
        gameboy.mem.joypad.b_pressed = Key::is_pressed(Key::X);
        gameboy.mem.joypad.start_pressed = Key::is_pressed(Key::Return);
        gameboy.mem.joypad.select_pressed = Key::is_pressed(Key::BackSpace);
        gameboy.mem.joypad.up_pressed = Key::is_pressed(Key::Up);
        gameboy.mem.joypad.down_pressed = Key::is_pressed(Key::Down);
        gameboy.mem.joypad.left_pressed = Key::is_pressed(Key::Left);
        gameboy.mem.joypad.right_pressed = Key::is_pressed(Key::Right);

        let mut cycles = 0;
        while cycles < CYCLES_PER_FRAME {
            cycles += gameboy.step();
        }
        if FPS_CYCLES_DEBUG {
            println!("Ran {} cycles that frame", cycles);
        }

        unsafe {
            screen_texture.update_from_pixels(&gameboy.gpu.get_sfml_frame(), sw, sh, 0, 0);
        }
        let mut screen_sprite = Sprite::with_texture(&screen_texture);
        screen_sprite.set_scale(sprite_scale);

        window.clear(Color::BLACK);
        window.draw(&screen_sprite);
        window.display();
    }
}
