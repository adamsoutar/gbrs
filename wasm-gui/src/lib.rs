use wasm_bindgen::prelude::*;
use web_sys::console;
use gbrs_core::cpu::Cpu;
use gbrs_core::{callbacks, callbacks::Callbacks, constants::*};

static mut CPU: Option<Cpu> = None;

#[wasm_bindgen]
pub fn create_gameboy() {
    console_error_panic_hook::set_once();
    
    unsafe {
        callbacks::set_callbacks(Callbacks {
            log: |log_str| console::log_1(&log_str.into()),
            save: |game_name, _rom_path, _save_data| 
                console::log_1(&format!("{} tried to save", game_name).into()),
            load: |game_name, _rom_path, expected_size| {
                console::log_1(&format!("{} tried to load", game_name).into());
                vec![0; expected_size as usize]
            }
        });

        CPU = Some(Cpu::from_rom_bytes(
            include_bytes!("../../roms/PokemonRed.gb").to_vec()
        ));
    }
}

#[wasm_bindgen]
pub fn step_one_frame() {
    unsafe {
        CPU.as_mut().unwrap().step_one_frame();
    }
}

#[wasm_bindgen]
pub fn get_finished_frame() -> Vec<usize> {
    let greyshade_frame = unsafe {
        CPU.as_mut().unwrap().gpu.finished_frame
    };
    // TODO: Re-use a buffer instead
    let mut int_frame = Vec::with_capacity(SCREEN_BUFFER_SIZE);

    for i in 0..SCREEN_BUFFER_SIZE {
        int_frame.push(greyshade_frame[i] as u8 as usize);
    }

    int_frame
}

#[wasm_bindgen]
pub fn set_control_state(a: bool, b: bool, up: bool, down: bool, 
    left: bool, right: bool, start: bool, select: bool) {
    unsafe {
        let cpu = CPU.as_mut().unwrap();
        cpu.mem.joypad.a_pressed = a;
        cpu.mem.joypad.b_pressed = b;
        cpu.mem.joypad.up_pressed = up;
        cpu.mem.joypad.down_pressed = down;
        cpu.mem.joypad.left_pressed = left;
        cpu.mem.joypad.right_pressed = right;
        cpu.mem.joypad.start_pressed = start;
        cpu.mem.joypad.select_pressed = select;
    }
}