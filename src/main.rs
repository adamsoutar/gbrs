pub mod gameboy;
pub mod gui;
use gameboy::cpu::Cpu;
use gui::run_gui;

fn main() {
    let processor = Cpu::from_rom("Tetris.gb".to_string());
    run_gui(processor);
}
