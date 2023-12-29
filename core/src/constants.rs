// "WRAM" is Work RAM, not Wave RAM
pub const WRAM_BANK_SIZE: usize = 4096;
pub const VRAM_BANK_SIZE: usize = 8192;
pub const HRAM_SIZE: usize = 127;
pub const OAM_SIZE: usize = 160;
pub const WAVE_RAM_SIZE: usize = 16;

// Excluding invisible areas such as those above and to
// the left of the screen
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

pub const SCREEN_BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;
pub const SCREEN_RGBA_SLICE_SIZE: usize = SCREEN_BUFFER_SIZE * 4;

pub const CLOCK_SPEED: usize = 4194304;
pub const DEFAULT_FRAME_RATE: usize = 60;

// The amount of sound samples we collect before firing them off for
// playback. This number is essentially guessed.
pub const SOUND_BUFFER_SIZE: usize = 1024;
pub const SOUND_SAMPLE_RATE: usize = 48000;
// The amount of APU step()s we should run before
// we sample for audio.
pub const APU_SAMPLE_CLOCKS: usize = CLOCK_SPEED / SOUND_SAMPLE_RATE;

// MBC_ROM_START is 0
pub const MBC_ROM_END: u16 = 0x7FFF;

pub const MBC_RAM_START: u16 = 0xA000;
pub const MBC_RAM_END: u16 = 0xBFFF;

pub const VRAM_START: u16 = 0x8000;
// For CGB BG Map Attribute Table
pub const VRAM_BG_MAP_START: u16 = 0x9800;
pub const VRAM_END: u16 = 0x9FFF;

pub const WRAM_LOWER_BANK_START: u16 = 0xC000;
pub const WRAM_LOWER_BANK_END: u16 = 0xCFFF;
pub const WRAM_UPPER_BANK_START: u16 = 0xD000;
pub const WRAM_UPPER_BANK_END: u16 = 0xDFFF;

pub const ECHO_RAM_START: u16 = 0xE000;
pub const ECHO_RAM_END: u16 = 0xFDFF;

pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;

pub const UNUSABLE_MEMORY_START: u16 = 0xFEA0;
pub const UNUSABLE_MEMORY_END: u16 = 0xFEFF;

pub const LINK_CABLE_SB: u16 = 0xFF01;
pub const LINK_CABLE_SC: u16 = 0xFF02;

pub const APU_START: u16 = 0xFF10;
pub const APU_END: u16 = 0xFF3F;

pub const WAVE_RAM_START: u16 = 0xFF30;
pub const WAVE_RAM_END: u16 = 0xFF3F;

pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;

pub const LCD_DATA_START: u16 = 0xFF40;
pub const LCD_DATA_END: u16 = 0xFF4E;

pub const CGB_PALETTE_DATA_START: u16 = 0xFF68;
pub const CGB_PALETTE_DATA_END: u16 = 0xFF6B;

pub const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;
pub const INTERRUPT_FLAG_ADDRESS: u16 = 0xFF0F;

pub mod gpu_timing {
    // Total line size incl. HBlank
    pub const HTOTAL: u16 = 456;

    // lx coordinate where Transfer begins
    pub const HTRANSFER_ON: u16 = 80;

    // Start of HBlank
    pub const HBLANK_ON: u16 = 252;

    // Total vertical lines incl. VBlank
    pub const VTOTAL:   u8 = 154;
    // Start of VBlank
    pub const VBLANK_ON: u8 = 144;

    // Number of CPU cycles it takes to do a DMA
    pub const DMA_CYCLES: u8 = 160;
}
