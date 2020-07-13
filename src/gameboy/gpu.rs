use crate::gameboy::constants::*;
use crate::gameboy::lcd::*;

pub struct Gpu {
    frame: [GreyShade; SCREEN_BUFFER_SIZE],

    // X and Y of background position
    scy: u8,
    scx: u8,

    // The scan-line Y co-ordinate
    ly: u8,
    status: LcdStatus,
    control: LcdControl
}

impl Gpu {
    pub fn raw_write (&mut self, raw_address: u16, value: u8) {
        match raw_address {
            0xFF40 => self.control = LcdControl::from(value),
            0xFF41 => self.status = LcdStatus::from(value),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            _ => panic!("Unsupported GPU write at {:#06x}", raw_address)
        }
    }
    pub fn raw_read (&self, raw_address: u16) -> u8 {
        match raw_address {
            0xFF40 => u8::from(self.control),
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
            status: LcdStatus::new(),
            control: LcdControl::new()
        }
    }
}
