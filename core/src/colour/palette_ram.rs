use crate::{cpu::EmulationTarget, memory::ram::Ram, combine_u8};
use super::colour::Colour;

fn palette_spec_read (address: u16, auto_increment: bool) -> u8 {
    // This should never be higher than 64 anyway, but let's be safe
    let lower_address = (address & 0b0001_1111) as u8;
    let auto_inc_bit = if auto_increment { 1 } else { 0 };
    lower_address | (auto_inc_bit << 7)
}

fn palette_spec_write (
    address: &mut u16, value: u8, auto_increment: &mut bool) {
    *auto_increment = (value & 0b1000_0000) > 0;
    *address = (value & 0b0001_1111) as u16;
}

fn palette_data_write (
    ram: &mut Ram, address: &mut u16, value: u8, auto_increment: bool) {
    ram.write(*address, value);
    if auto_increment {
        *address = (*address + 1) % 64;
    }
}

pub struct PaletteRam {
    // If this is false, we're a DMG.
    cgb_features: bool,
    bg_palette_ram: Ram,
    bg_address: u16,
    bg_auto_increment: bool,
    obj_palette_ram: Ram,
    obj_address: u16,
    obj_auto_increment: bool
}

impl PaletteRam {
    fn read_colour (&self, ram: &Ram, address: u16) -> Colour {
        let col0 = ram.read(address);
        let col1 = ram.read(address + 1);
        Colour::from_16_bit_colour(combine_u8!(col1, col0))
    }

    pub fn get_obj_palette_colour (&self, palette_id: u16, colour_id: u16) -> Colour {
        let base_offset = 8 * palette_id;
        self.read_colour(&self.obj_palette_ram, base_offset + colour_id * 2)
    }

    pub fn raw_read(&self, address: u16) -> u8 {
        if !self.cgb_features { return 0xFF; }

        match address {
            0xFF68 => palette_spec_read(self.bg_address, self.bg_auto_increment),
            0xFF69 => self.bg_palette_ram.read(self.bg_address),

            0xFF6A => palette_spec_read(self.obj_address, self.obj_auto_increment),
            0xFF6B => self.obj_palette_ram.read(self.obj_address),

            _ => panic!("CGB Palette RAM read at {:#06x}", address)
        }
    }

    pub fn raw_write(&mut self, address: u16, value: u8) {
        if !self.cgb_features { return; }

        match address {
            0xFF68 => palette_spec_write(&mut self.bg_address, value, &mut self.bg_auto_increment),
            0xFF69 => palette_data_write(
                &mut self.bg_palette_ram,
                &mut self.bg_address,
                value,
                self.bg_auto_increment
            ),

            0xFF6A => palette_spec_write(&mut self.obj_address, value, &mut self.obj_auto_increment),
            0xFF6B => palette_data_write(
                &mut self.obj_palette_ram,
                &mut self.obj_address,
                value,
                self.obj_auto_increment
            ),

            _ => panic!("CGB Palette RAM write at {:#06x} (value: {:#04x})", address, value)
        }
    }

    pub fn new(target: &EmulationTarget) -> PaletteRam {
        PaletteRam {
            cgb_features: target.has_cgb_features(),
            // All background colours are white at boot
            bg_palette_ram: Ram::with_filled_value(64, 0xFF),
            bg_address: 0,
            bg_auto_increment: false,
            // Object memory is garbage on boot, but slot 0 is always 0
            obj_palette_ram: Ram::new(64),
            obj_address: 0,
            obj_auto_increment: false
        }
    }
}
