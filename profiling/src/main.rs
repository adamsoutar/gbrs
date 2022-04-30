use std::env;
use std::time::SystemTime;

use gbrs_core::cpu::Cpu;

const RUNS: u128 = 5000;

fn main() {
    let rom_path = env::args().nth(1).expect("Pass a ROM path as an argument");
    let mut processor = Cpu::from_rom_file(rom_path);

    // Just run the CPU forever so we can profile hot areas of emulation.
    let mut harness_total = 0;
    for _ in 0..RUNS {
        let now = SystemTime::now();

        processor.step_one_frame();

        let time = now.elapsed().unwrap().as_micros();
        harness_total += time;
    }

    println!("Average execution time across {} runs: {} microseconds", RUNS, harness_total / RUNS);
}
