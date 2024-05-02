use crate::log;

#[derive(Debug, PartialEq)]
pub enum CgbDmaType {
    GeneralPurpose,
    HBlank
}

pub struct CgbDmaConfig {
    pub source: u16,
    pub dest: u16,
    pub dma_type: CgbDmaType,
    pub bytes_left: u16,
}

impl CgbDmaConfig {
    pub fn set_config_byte(&mut self, value: u8) {
        self.dma_type = if value & 0x80 == 0x80 {
            CgbDmaType::HBlank
        } else {
            CgbDmaType::GeneralPurpose
        };
        self.bytes_left = (value & 0x7F) as u16 * 0x10 + 1;
    }
    pub fn get_config_byte(&self) -> u8 {
        // let top_bit = match self.dma_type {
        //     CgbDmaType::HBlank => 0x80,
        //     CgbDmaType::GeneralPurpose => 0x00
        // };
        // TODO: Because we don't currently support HBlank transfer, we will
        //   always return a set bit 7, which means 'no transfer active'
        0x80 | ((self.bytes_left - 1) / 0x10) as u8
    }

    pub fn get_source_upper(&self) -> u8 {
        (self.source >> 8) as u8
    }
    pub fn get_source_lower(&self) -> u8 {
        (self.source & 0xFF) as u8
    }
    pub fn set_source_upper(&mut self, value: u8) {
        self.source = (self.source & 0x00FF) | ((value as u16) << 8);
    }
    pub fn set_source_lower(&mut self, value: u8) {
        // Lower 4 bits of address are ignored
        self.source = (self.source & 0xFF00) | ((value & 0xF0) as u16);
    }

    pub fn get_dest_upper(&self) -> u8 {
        (self.dest >> 8) as u8
    }
    pub fn get_dest_lower(&self) -> u8 {
        (self.dest & 0xFF) as u8
    }
    pub fn set_dest_upper(&mut self, value: u8) {
        // This algo makes sure that the destination address is in the range
        // 0x8000 - 0x9FFF, ensuring that the destianation is in VRAM.
        self.dest =
            // Keep lower byte
            (self.dest & 0x00FF) |
            // Set upper byte
            (((
                // Ignore upper 3 bits of value, making range 0x0000 - 0x1FFF
                (value & 0x1F)
                // Set the top bit, adding 0x8000
                    | 0x80) as u16
            ) << 8);
    }
    pub fn set_dest_lower(&mut self, value: u8) {
        // Lower 4 bits of address are ignored
        self.dest = (self.dest & 0xFF00) | ((value & 0xF0) as u16);
    }

    pub fn new() -> CgbDmaConfig {
        CgbDmaConfig {
            source: 0,
            dest: 0,
            dma_type: CgbDmaType::GeneralPurpose,
            bytes_left: 0,
        }
    }
}
