pub const WRAM_SIZE: usize = 8192;

// TODO: Switchable ROM
pub const ROM_START: u16 = 0x0000;
pub const ROM_END: u16 = 0x3FFF;

// TODO: VRAM etc

pub const WRAM_START: u16 = 0xC000;
pub const WRAM_END: u16 = 0xDFFF;

pub const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;
pub const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;
