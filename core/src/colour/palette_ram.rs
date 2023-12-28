use crate::{cpu::EmulationTarget, memory::ram::Ram};

pub struct PaletteRam {
    // If this is false, we're a DMG.
    cgb_features: bool,
    bg_palette_ram: Ram,
    obj_palette_ram: Ram,
}

impl PaletteRam {
    pub fn raw_read(&self, address: u16) -> u8 {
        if !self.cgb_features { return 0xFF; }

        match address {
            _ => unimplemented!("CGB Palette RAM read at {:#06x}", address)
        }
    }

    pub fn raw_write(&mut self, address: u16, value: u8) {
        if !self.cgb_features { return; }

        match address {
            _ => unimplemented!("CGB Palette RAM write at {:#06x} (value: {:#04x})", address, value)
        }
    }

    pub fn new(target: &EmulationTarget) -> PaletteRam {
        PaletteRam {
            cgb_features: target.has_cgb_features(),
            bg_palette_ram: Ram::new(64),
            obj_palette_ram: Ram::new(64)
        }
    }
}
