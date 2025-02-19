use super::ram::Ram;
use crate::colour::bg_map_attributes::BgMapAttributeTable;
use crate::constants::*;

pub struct VRam {
    cgb_features: bool,
    memory: Ram,
    pub bank: u16,
    pub bg_map_attributes: BgMapAttributeTable,
}

impl VRam {
    pub fn read_arbitrary_bank(&self, bank: u16, address: u16) -> u8 {
        let relative_address = address - VRAM_START;
        self.memory
            .read(bank * VRAM_BANK_SIZE as u16 + relative_address)
    }

    pub fn raw_read(&self, address: u16) -> u8 {
        if self.bank == 1 && address > VRAM_BG_MAP_START {
            return self.bg_map_attributes.read(address - VRAM_BG_MAP_START);
        }

        let relative_address = address - VRAM_START;
        self.memory
            .read(self.bank * VRAM_BANK_SIZE as u16 + relative_address)
    }

    pub fn raw_write(&mut self, address: u16, value: u8) {
        if self.bank == 1 && address > VRAM_BG_MAP_START {
            // Attribute table
            self.bg_map_attributes
                .write(address - VRAM_BG_MAP_START, value);
            return;
        }

        let relative_address = address - VRAM_START;
        self.memory
            .write(self.bank * VRAM_BANK_SIZE as u16 + relative_address, value)
    }

    pub fn bank_write(&mut self, value: u8) {
        if !self.cgb_features {
            return;
        }
        self.bank = value as u16 & 0x01;
    }

    pub fn new(cgb_features: bool) -> VRam {
        VRam {
            cgb_features,
            memory: Ram::new(VRAM_BANK_SIZE * 2),
            bank: 0,
            bg_map_attributes: BgMapAttributeTable::new(),
        }
    }
}
