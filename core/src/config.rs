// Config for creating CPUs
// This helps with ports
use crate::memory::rom::Rom;

#[derive(Clone)]
pub struct Config {
    pub sound_buffer_size: usize,
    pub sound_sample_rate: usize,
    pub rom: Rom,
}
