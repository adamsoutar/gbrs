pub mod gameboy;
use gameboy::memory::memory::Memory;

fn main() {
    let mem = Memory::new("Tetris.gb".to_string());
}
