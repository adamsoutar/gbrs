use std::env;
use std::time::SystemTime;

use gbrs_core::{
    config::Config,
    constants::{SOUND_BUFFER_SIZE, SOUND_SAMPLE_RATE},
    cpu::Cpu,
    memory::rom::Rom,
};

const RUNS: u128 = 5000;

fn main() {
    let rom_path = env::args().nth(1).expect("Pass a ROM path as an argument");
    let mut processor = Cpu::from_config(Config {
        rom: Rom::from_file(&rom_path),
        sound_buffer_size: SOUND_BUFFER_SIZE,
        sound_sample_rate: SOUND_SAMPLE_RATE,
    });

    // Just run the CPU forever so we can profile hot areas of emulation.
    let mut harness_total = 0;
    for _ in 0..RUNS {
        let now = SystemTime::now();

        processor.step_one_frame();

        let time = now.elapsed().unwrap().as_micros();
        harness_total += time;
    }

    println!(
        "Average execution time across {} runs: {} microseconds",
        RUNS,
        harness_total / RUNS
    );
}
