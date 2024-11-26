use gbrs_core::cpu::Cpu;

use sfml::window::joystick::*;
use sfml::window::*;

// TODO: Mappings for:
//   - Xbox 360
//   - Xbox One
//   - DualShock 5
//   - The above on Windows
#[allow(dead_code)]
mod ps4 {
    // These are ascertained through experimentation with a wired DualShock 4
    // on macOS.
    use sfml::window::joystick::Axis;

    pub const X: u32 = 1;
    pub const SQUARE: u32 = 0;
    pub const TRIANGLE: u32 = 3;
    pub const CIRCLE: u32 = 2;

    pub const START: u32 = 9;
    pub const SHARE: u32 = 8;
    pub const TOUCHPAD: u32 = 13;

    pub const LEFT_STICK_X: Axis = Axis::X;
    pub const LEFT_STICK_Y: Axis = Axis::Y;
    pub const RIGHT_STICK_X: Axis = Axis::Z;
    pub const RIGHT_STICK_Y: Axis = Axis::R;
    pub const DPAD_X: Axis = Axis::PovX;
    pub const DPAD_Y: Axis = Axis::PovY;

    // DualShock 4 axes go from -100 to +100
    pub const DEADZONE: f32 = 25.;
}

pub fn update_joypad_state(gameboy: &mut Cpu) {
    // TODO: Raise the joypad interrupt
    gameboy.mem.joypad.a_pressed =
        key(Key::X) || joy(ps4::X) || joy(ps4::CIRCLE);

    gameboy.mem.joypad.b_pressed =
        key(Key::Z) || joy(ps4::SQUARE) || joy(ps4::TRIANGLE);

    gameboy.mem.joypad.start_pressed = key(Key::Enter) || joy(ps4::START);

    gameboy.mem.joypad.select_pressed =
        key(Key::Backspace) || joy(ps4::TOUCHPAD) || joy(ps4::SHARE);

    gameboy.mem.joypad.up_pressed = key(Key::Up)
        || axis(ps4::LEFT_STICK_Y, false)
        || axis(ps4::DPAD_Y, true);

    gameboy.mem.joypad.down_pressed = key(Key::Down)
        || axis(ps4::LEFT_STICK_Y, true)
        || axis(ps4::DPAD_Y, false);

    gameboy.mem.joypad.left_pressed = key(Key::Left)
        || axis(ps4::LEFT_STICK_X, false)
        || axis(ps4::DPAD_X, false);

    gameboy.mem.joypad.right_pressed = key(Key::Right)
        || axis(ps4::LEFT_STICK_X, true)
        || axis(ps4::DPAD_X, true);
}

fn key(key: Key) -> bool {
    Key::is_pressed(key)
}
fn joy(button: u32) -> bool {
    for i in 0..joystick::COUNT {
        if joystick::is_button_pressed(i, button) {
            return true;
        }
    }

    false
}
fn axis(target: Axis, positive: bool) -> bool {
    for i in 0..joystick::COUNT {
        let mut val = joystick::axis_position(i, target);
        if !positive {
            val = -val
        }

        if val > 100. - ps4::DEADZONE {
            return true;
        }
    }

    false
}
