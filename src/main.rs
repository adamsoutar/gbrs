pub mod gameboy;
pub mod gui;
use gameboy::cpu::Cpu;
use gui::run_gui;

fn main() {
    // let processor = Cpu::from_rom("roms/individual/02-interrupts.gb".to_string());
    let processor = Cpu::from_rom("roms/dmg-acid2.gb".to_string());
    // let processor = Cpu::from_rom("roms/Tetris.gb".to_string());
    run_gui(processor);
}
