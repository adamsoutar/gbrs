use crate::gameboy::constants::*;
use crate::gameboy::memory::ram::Ram;
use crate::gameboy::memory::rom::Rom;

pub struct Memory {
    pub rom: Rom,
    pub wram: Ram
}

impl Memory {
    pub fn read (&self, address: u16) -> u8 {
        match address {
            
        }
    }

    pub fn new (rom_path: String) -> Memory {
        Memory {
            rom: Rom::from_file(rom_path),
            wram: Ram::new(WRAM_SIZE)
        }
    }
}
