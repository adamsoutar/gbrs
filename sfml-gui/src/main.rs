pub mod gui;
pub mod control;

use std::env;

use gbrs_core::cpu::Cpu;
use gui::run_gui;

fn main() {
    let rom_path = env::args().nth(1).expect("Pass a ROM path as an argument");
    let processor = Cpu::from_rom_file(rom_path);
    run_gui(processor);
}
