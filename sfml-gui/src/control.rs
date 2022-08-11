use gbrs_core::cpu::Cpu;

use sfml::window::*;
use sfml::window::joystick::*;

// These are ascertained through experimentation with a wired DualShock 4
// on macOS.
// TODO: Mappings for:
//   - Xbox 360
//   - Xbox One
//   - DualShock 5
//   - The above on Windows
const PS4_X: u32 = 1;
const PS4_SQUARE: u32 = 0;
const PS4_TRIANGLE: u32 = 3;
const PS4_CIRCLE: u32 = 2;
const PS4_START: u32 = 9;
const PS4_SHARE: u32 = 8;
const PS4_TOUCHPAD: u32 = 13;

const PS4_LEFT_STICK_X: Axis = Axis::X;
const PS4_LEFT_STICK_Y: Axis = Axis::Y;
const PS4_RIGHT_STICK_X: Axis = Axis::Z;
const PS4_RIGHT_STICK_Y: Axis = Axis::R;
const PS4_DPAD_X: Axis = Axis::PovX;
const PS4_DPAD_Y: Axis = Axis::PovY;
// DualShock 4 axes go from -100 to +100
const PS4_DEADZONE: f32 = 25.;

pub fn update_joypad_state (gameboy: &mut Cpu) {
    // TODO: Raise the joypad interrupt
    gameboy.mem.joypad.a_pressed = 
      key(Key::X) || joy(PS4_X) || joy(PS4_CIRCLE);
    
    gameboy.mem.joypad.b_pressed = 
      key(Key::Z) || joy(PS4_SQUARE) || joy(PS4_TRIANGLE);
    
    gameboy.mem.joypad.start_pressed = 
      key(Key::Return) || joy(PS4_START);
    
    gameboy.mem.joypad.select_pressed = 
      key(Key::BackSpace) || joy(PS4_TOUCHPAD) || joy(PS4_SHARE);
    
    gameboy.mem.joypad.up_pressed = 
      key(Key::Up) || axis(PS4_LEFT_STICK_Y, false) || axis(PS4_DPAD_Y, true);
    
    gameboy.mem.joypad.down_pressed = 
      key(Key::Down) || axis(PS4_LEFT_STICK_Y, true) || axis(PS4_DPAD_Y, false);
    
    gameboy.mem.joypad.left_pressed = 
      key(Key::Left) || axis(PS4_LEFT_STICK_X, false) || axis(PS4_DPAD_X, false);
    
    gameboy.mem.joypad.right_pressed = 
      key(Key::Right) || axis(PS4_LEFT_STICK_X, true) || axis(PS4_DPAD_X, true);
}

fn key (key: Key) -> bool {
  Key::is_pressed(key)
}
fn joy (button: u32) -> bool {
  for i in 0..joystick::COUNT {
    if joystick::is_button_pressed(i, button) { return true }
  }
  
  false
}
fn axis (target: Axis, positive: bool) -> bool {
  for i in 0..joystick::COUNT {
    let mut val = joystick::axis_position(i, target);
    if !positive { val = -val }
    
    if val > 100. - PS4_DEADZONE { return true }
  }
  
  false
}
