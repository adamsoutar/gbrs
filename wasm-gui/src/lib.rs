use gbrs_core::config::Config;
use gbrs_core::constants;
use gbrs_core::cpu::Cpu;
use gbrs_core::memory::rom::Rom;
use gbrs_core::{callbacks, callbacks::Callbacks, constants::*};
use wasm_bindgen::prelude::*;
use web_sys::{console, window, Storage};

static mut CPU: Option<Cpu> = None;

fn local_storage() -> Storage {
    window().unwrap().local_storage().unwrap().unwrap()
}

#[wasm_bindgen]
pub fn create_gameboy() {
    console_error_panic_hook::set_once();

    unsafe {
        callbacks::set_callbacks(Callbacks {
            log: |log_str| console::log_1(&log_str.into()),
            save: |game_name, _rom_path, save_data| {
                let data_string = base64::encode(save_data);
                local_storage()
                    .set_item(game_name, &data_string)
                    .expect("Failed to save in localStorage");
            },
            load: |game_name, _rom_path, expected_size| {
                let optional_data_string = local_storage()
                    .get_item(game_name)
                    .expect("Failed to read save in localStorage");

                if let Some(data_string) = optional_data_string {
                    // This game already has save data in this browser
                    let loaded_data = base64::decode(data_string).unwrap();
                    if loaded_data.len() == expected_size {
                        return loaded_data;
                    }
                }
                // Else we've not run this game before
                vec![0; expected_size as usize]
            },
        });

        CPU = Some(Cpu::from_config(Config {
            sound_buffer_size: constants::SOUND_BUFFER_SIZE,
            sound_sample_rate: constants::SOUND_SAMPLE_RATE,
            rom: Rom::from_bytes(
                include_bytes!("../../roms/dmg-acid2.gb").to_vec(),
            ),
        }));
    }
}

#[wasm_bindgen]
pub fn step_one_frame() {
    unsafe {
        CPU.as_mut().unwrap().step_one_frame();
    }
}

#[wasm_bindgen]
pub fn get_finished_frame() -> Vec<String> {
    let frame = unsafe { CPU.as_mut().unwrap().gpu.finished_frame };
    // TODO: Re-use a buffer instead
    let mut int_frame = Vec::with_capacity(SCREEN_BUFFER_SIZE);

    for i in 0..SCREEN_BUFFER_SIZE {
        int_frame.push(format!(
            "rgb({},{},{})",
            frame[i].red, frame[i].green, frame[i].blue
        ));
    }

    int_frame
}

#[wasm_bindgen]
pub fn set_control_state(
    a: bool,
    b: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    start: bool,
    select: bool,
) {
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
