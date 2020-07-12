use crate::gameboy::constants::*;

#[derive(Clone, Copy)]
pub enum GreyShade {
    White,
    LightGrey,
    DarkGrey,
    Black
}

#[derive(Clone, Copy)]
pub struct LcdStatus {
    lyc: bool,
    oam_interrupt: bool,
    vblank_interrupt: bool,
    hblank_interrupt: bool,
    coincidence_flag: bool,
    mode_flag: u8
}
impl LcdStatus {
    pub fn new () -> LcdStatus {
        LcdStatus::from(0)
    }
}
impl From<u8> for LcdStatus {
    fn from(n: u8) -> LcdStatus {
        LcdStatus {
            lyc: (n & 0b1000000) == 0b1000000,
            oam_interrupt: (n & 0b100000) == 0b100000,
            vblank_interrupt: (n & 0b10000) == 0b10000,
            hblank_interrupt: (n & 0b1000) == 0b1000,
            coincidence_flag: (n & 0b100) == 0b100,
            mode_flag: n & 0b11
        }
    }
}
impl From<LcdStatus> for u8 {
    fn from(lcd: LcdStatus) -> u8 {
        lcd.mode_flag |
        (lcd.coincidence_flag as u8) << 2 |
        (lcd.hblank_interrupt as u8) << 3 |
        (lcd.vblank_interrupt as u8) << 4 |
        (lcd.oam_interrupt as u8) << 5 |
        (lcd.lyc as u8) << 6
    }
}

pub struct Gpu {
    frame: [GreyShade; SCREEN_BUFFER_SIZE],

    // X and Y of background position
    scy: u8,
    scx: u8,

    // The scan-line Y co-ordinate
    ly: u8,
    status: LcdStatus
}

impl Gpu {
    pub fn raw_write (&mut self, raw_address: u16, value: u8) {
        match raw_address {
            0xFF41 => self.status = LcdStatus::from(value),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            _ => panic!("Unsupported GPU write at {:#06x}", raw_address)
        }
    }
    pub fn raw_read (&self, raw_address: u16) -> u8 {
        match raw_address {
            0xFF41 => u8::from(self.status),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            _ => panic!("Unsupported GPU read at {:#06x}", raw_address)
        }
    }

    pub fn get_sfml_frame (&self) -> [u8; SCREEN_RGBA_SLICE_SIZE] {
        let mut out_array = [0; SCREEN_RGBA_SLICE_SIZE];
        for i in 0..SCREEN_BUFFER_SIZE {
            let start = i * 4;
            match &self.frame[i] {
                GreyShade::White => {
                    out_array[start] = 0xDD;
                    out_array[start + 1] = 0xDD;
                    out_array[start + 2] = 0xDD;
                    out_array[start + 3] = 0xFF;
                },
                GreyShade::LightGrey => {
                    out_array[start] = 0xAA;
                    out_array[start + 1] = 0xAA;
                    out_array[start + 2] = 0xAA;
                    out_array[start + 3] = 0xFF;
                },
                GreyShade::DarkGrey => {
                    out_array[start] = 0x88;
                    out_array[start + 1] = 0x88;
                    out_array[start + 2] = 0x88;
                    out_array[start + 3] = 0xFF;
                },
                GreyShade::Black => {
                    out_array[start] = 0x55;
                    out_array[start + 1] = 0x55;
                    out_array[start + 2] = 0x55;
                    out_array[start + 3] = 0xFF;
                }
            }
        }
        out_array
    }

    pub fn new () -> Gpu {
        Gpu {
            frame: [GreyShade::White; SCREEN_BUFFER_SIZE],
            scy: 0, scx: 0, ly: 0,
            status: LcdStatus::new()
        }
    }
}
