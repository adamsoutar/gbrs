use crate::gameboy::constants::*;
use crate::gameboy::memory::ram::Ram;

// Audio processing unit
// NOTE: Max APU frequency seems to be 131072 Hz
pub struct APU {
    pub stereo_left_volume: f32,
    pub stereo_right_volume: f32,

    pub stereo_channel_control: u8,

    pub sound_on_register: u8,

    pub wave_ram: Ram,

    ch3_on: u8,
    ch3_len: u8,
    ch3_out_level: u8,
    ch3_nr33: u8,
    ch3_nr34: u8,
}

impl APU {
    pub fn read (&self, address: u16) -> u8 {
        match address {
            0xFF24 => self.serialise_nr50(),
            0xFF25 => self.stereo_channel_control,
            0xFF26 => self.sound_on_register,

            // Sound Channel 3
            0xFF1A => self.ch3_on,
            0xFF1B => self.ch3_len,
            0xFF1C => self.ch3_out_level,
            0xFF1D => self.ch3_nr33,
            0xFF1E => self.ch3_nr34,

            WAVE_RAM_START ..= WAVE_RAM_END => self.wave_ram.read(address - WAVE_RAM_START),
            _ => 0 //panic!("Unknown read {:#06x} in APU", address)
        }
    }

    pub fn write (&mut self, address: u16, value: u8) {
        match address {
            0xFF24 => self.deserialise_nr50(value),
            0xFF25 => self.stereo_channel_control = value,
            0xFF26 => self.sound_on_register = value,

            // Sound Channel 3
            0xFF1A => self.ch3_on = value,
            0xFF1B => self.ch3_len = value,
            0xFF1C => self.ch3_out_level = value,
            0xFF1D => self.ch3_nr33 = value,
            0xFF1E => self.ch3_nr34 = value,

            WAVE_RAM_START ..= WAVE_RAM_END => self.wave_ram.write(address - WAVE_RAM_START, value),
            _ => {} //println!("Unknown write {:#06x} (value: {:#04}) in APU", address, value)
        }
    }

    // NOTE: These functions don't take into account the
    //       Vin output flags. That feature is unused in all
    //       commercial Gameboy games, so we ignore it.
    fn deserialise_nr50 (&mut self, nr50: u8) {
        let right_vol = nr50 & 0b111;
        let left_vol = (nr50 & 0b111_0_000) >> 4;

        self.stereo_left_volume = (left_vol as f32) / 7.;
        self.stereo_right_volume = (right_vol as f32) / 7.;
    }
    fn serialise_nr50 (&self) -> u8 {
        // These might turn out 1 level too low because of float flooring
        // TODO: Test this
        let right_vol = (self.stereo_right_volume * 7.) as u8;
        let left_vol = (self.stereo_left_volume * 7.) as u8;

        (left_vol << 4) & right_vol
    }

    pub fn new () -> APU {
        APU {
            // These might be meant to start 0, not sure
            stereo_left_volume: 1.,
            stereo_right_volume: 1.,
            stereo_channel_control: 0,
            sound_on_register: 0,

            wave_ram: Ram::new(WAVE_RAM_SIZE),

            ch3_on: 0,
            ch3_len: 0,
            ch3_nr33: 0,
            ch3_nr34: 0,
            ch3_out_level: 0
        }
    }
}
