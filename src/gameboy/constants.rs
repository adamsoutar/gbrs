pub const WRAM_SIZE: usize = 8192;
pub const VRAM_SIZE: usize = 8192;

// Excluding invisible areas such as those above and to
// the left of the screen
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub const SCREEN_VEC_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
pub const SCREEN_RGBA_SLICE_SIZE: usize = SCREEN_VEC_SIZE * 4;

pub const CLOCK_SPEED: usize = 4194304;
pub const FRAME_RATE: usize = 60;
pub const CYCLES_PER_FRAME: usize = CLOCK_SPEED / FRAME_RATE;

// TODO: Switchable ROM
pub const ROM_START: u16 = 0x0000;
pub const ROM_END: u16 = 0x3FFF;

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;

pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;

pub const WRAM_START: u16 = 0xC000;
pub const WRAM_END: u16 = 0xDFFF;

pub const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;
pub const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;
