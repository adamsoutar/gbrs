use crate::constants::*;
use super::ram::Ram;

pub struct VRam {
    cgb_features: bool,
    memory: Ram,
    pub bank: u16
}

impl VRam {
    pub fn raw_read(&self, address: u16) -> u8 {
        let relative_address = address - VRAM_START;
        self.memory.read(self.bank * VRAM_BANK_SIZE as u16 + relative_address)
    }

    pub fn raw_write(&mut self, address: u16, value: u8) {
        let relative_address = address - VRAM_START;
        self.memory.write(self.bank * VRAM_BANK_SIZE as u16 + relative_address, value)
    }

    pub fn bank_write(&mut self, value: u8) {
        if !self.cgb_features { return; }
        self.bank = value as u16 & 0x01;
    }

    pub fn new (cgb_features: bool) -> VRam {
        VRam {
            cgb_features,
            memory: Ram::new(VRAM_BANK_SIZE * 2),
            bank: 0
        }
    }
}
