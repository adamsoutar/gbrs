#[derive(Debug, PartialEq)]
pub enum CgbDmaType {
    GeneralPurpose,
    HBlank
}

pub struct CgbDmaConfig {
    pub source: u16,
    pub dest: u16,
    pub dma_type: CgbDmaType,
    pub bytes_copied: u16,
    pub bytes_left: u16,
    pub transfer_done: bool
}

impl CgbDmaConfig {
    pub fn set_config_byte(&mut self, value: u8) {
        self.transfer_done = false;
        self.dma_type = if value & 0x80 == 0x80 {
            CgbDmaType::HBlank
        } else {
            CgbDmaType::GeneralPurpose
        };
        self.bytes_left = ((value & 0x7F) + 1) as u16 * 0x10;
        self.bytes_copied = 0;
    }
    pub fn get_config_byte(&self) -> u8 {
        if self.transfer_done {
            return 0xFF
        }
        // TODO: Not sure this is quite the correct calculation
        ((self.bytes_left / 0x10) - 1) as u8
    }

    pub fn is_hblank_dma(&self) -> bool {
        self.dma_type == CgbDmaType::HBlank
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
            bytes_copied: 0,
            transfer_done: false
        }
    }
}
