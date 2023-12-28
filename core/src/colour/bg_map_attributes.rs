// Data pertaining to rendering coloured background/window tiles
// Defined by writing to VRAM bank 1 0x9800 to 0x9FFF

const BG_MAP_ATTRIBUTE_TABLE_SIZE: usize = 0x7FF; // 0x9FFF - 0x9800

#[derive(Clone, Copy)]
pub struct BgMapAttributeEntry {
    pub priority: bool,
    pub y_flip: bool,
    pub x_flip: bool,
    // This is an unused bit, but the hardware keeps track of it
    // Games could use it for their own unusual hackery
    pub bit_four: bool,
    pub vram_bank: u8, // Either 0 or 1
    pub palette: u8,
}

impl BgMapAttributeEntry {
    pub fn as_u8(&self) -> u8 {
        let mut val = 0;
        if self.priority { val |= 0b1000_0000; }
        if self.y_flip { val |= 0b0100_0000; }
        if self.x_flip { val |= 0b0010_0000; }
        if self.bit_four { val |= 0b0001_0000; }
        val |= self.vram_bank << 3;
        val |= self.palette & 0b0000_0111;
        val
    }

    pub fn from_u8(val: u8) -> BgMapAttributeEntry {
        BgMapAttributeEntry {
            priority: (0b1000_0000 & val) > 0,
            y_flip: (0b0100_0000 & val) > 0,
            x_flip: (0b0010_0000 & val) > 0,
            bit_four: (0b0001_0000 & val) > 0,
            vram_bank: (0b0000_1000 & val) >> 3,
            palette: 0b0000_0111 & val
        }
    }

    pub fn new () -> BgMapAttributeEntry {
        BgMapAttributeEntry {
            priority: false,
            y_flip: false,
            x_flip: false,
            bit_four: false,
            vram_bank: 0,
            palette: 0
        }
    }
}

pub struct BgMapAttributeTable {
    entries: [BgMapAttributeEntry; BG_MAP_ATTRIBUTE_TABLE_SIZE]
}

impl BgMapAttributeTable {
    pub fn read (&self, address: u16) -> u8 {
        self.entries[address as usize].as_u8()
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.entries[address as usize] = BgMapAttributeEntry::from_u8(value);
    }

    pub fn new () -> BgMapAttributeTable {
        BgMapAttributeTable {
            entries: [BgMapAttributeEntry::new(); BG_MAP_ATTRIBUTE_TABLE_SIZE]
        }
    }
}
