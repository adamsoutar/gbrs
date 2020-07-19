pub mod gameboy;
pub mod gui;

use std::env;

use gameboy::cpu::Cpu;
use gui::run_gui;

fn main() {
    let rom_path = env::args().nth(1).expect("Pass a ROM path as an argument");
    let processor = Cpu::from_rom(rom_path);
    run_gui(processor);
}
