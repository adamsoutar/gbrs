pub mod gameboy;
use gameboy::cpu::Cpu;


fn main() {
    let mut processor = Cpu::from_rom("Tetris.gb".to_string());
    processor.run();
}
