pub mod gui;
pub mod control;

use std::env;

use gbrs_core::memory::rom::Rom;
use gbrs_core::config::Config;
use gbrs_core::cpu::Cpu;
use gui::run_gui;

// TODO: Get these from an SFML audio device
const SOUND_BUFFER_SIZE: usize = 1024;
const SOUND_SAMPLE_RATE: usize = 48000;

fn main() {
    let rom_path = env::args().nth(1).expect("Pass a ROM path as an argument");
    let processor = Cpu::from_config(Config {
        sound_buffer_size: SOUND_BUFFER_SIZE,
        sound_sample_rate: SOUND_SAMPLE_RATE,
        rom: Rom::from_file(&rom_path)
    });
    run_gui(processor);
}
