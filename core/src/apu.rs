use crate::callbacks::CALLBACKS;
use crate::constants::*;
use crate::memory::ram::Ram;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

// Audio processing unit
// NOTE: Max APU frequency seems to be 131072 Hz
pub struct APU {
    pub stereo_left_volume: f32,
    pub stereo_right_volume: f32,

    pub stereo_channel_control: u8,

    pub sound_on_register: u8,

    pub wave_ram: Ram,

    // ch3_on: u8,
    // ch3_len: u8,
    // ch3_out_level: u8,
    // ch3_freq_low: u8,
    // ch3_freq_high: u8,
    pub sin_counter: f64,
    pub sample_counter: usize,
    // This could be a Vec that we check len() against, but we can save the 
    // allocation because we know the size it's always going to be.
    pub buffer: [i16; SOUND_BUFFER_SIZE],
    pub buffer_idx: usize
}

impl APU {
    pub fn step (&mut self) {
        self.sample_counter += 1;
        self.sin_counter += 0.0006;

        if self.sample_counter == APU_SAMPLE_CLOCKS {
            self.sample_counter = 0;
            self.sample();
        }
    }

    pub fn sample (&mut self) {
        let made_up_sample = ((self.sin_counter).sin() * 1000.).floor() as i16;
        self.buffer[self.buffer_idx] = made_up_sample;
        self.buffer_idx += 1;

        if self.buffer_idx == SOUND_BUFFER_SIZE {
            self.buffer_idx = 0;
            unsafe {
                (CALLBACKS.play_sound)(&self.buffer)
            }  
        }
    }

    pub fn read (&self, address: u16) -> u8 {
        match address {
            0xFF24 => self.serialise_nr50(),
            0xFF25 => self.stereo_channel_control,
            0xFF26 => self.sound_on_register,

            // Sound Channel 3
            // 0xFF1A => self.ch3_on,
            // 0xFF1B => self.ch3_len,
            // 0xFF1C => self.ch3_out_level,
            // 0xFF1D => self.ch3_freq_low,
            // 0xFF1E => self.ch3_freq_high,

            WAVE_RAM_START ..= WAVE_RAM_END => self.wave_ram.read(address - WAVE_RAM_START),
            _ => 0 //panic!("Unknown read {:#06x} in APU", address)
        }
    }

    pub fn write (&mut self, address: u16, value: u8) {
        match address {
            0xFF24 => self.deserialise_nr50(value),
            0xFF25 => self.stereo_channel_control = value,
            0xFF26 => self.sound_on_register = value,

            // Sound Channel 1
            0xFF10..=0xFF14 => {
                // log!("Wrote to Sound Channel 1")
            },

            // Sound Channel 2
            0xFF16..=0xFF19 => {
                // log!("Wrote to Sound Channel 2")
            },

            // Sound Channel 3
            0xFF1A..=0xFF1E => {
                // log!("Wrote to Sound Channel 3")
            },
            // 0xFF1A => self.ch3_on = value,
            // 0xFF1B => self.ch3_len = value,
            // 0xFF1C => self.ch3_out_level = value,
            // 0xFF1D => self.ch3_freq_low = value,
            // 0xFF1E => self.ch3_freq_high = value,

            // Sound Channel 4
            0xFF20..=0xFF23 => {
                // log!("Wrote to Sound Channel 4")
            },

            WAVE_RAM_START ..= WAVE_RAM_END => self.wave_ram.write(address - WAVE_RAM_START, value),
            _ => {} //log!("Unknown write {:#06x} (value: {:#04}) in APU", address, value)
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

            // ch3_on: 0,
            // ch3_len: 0,
            // ch3_nr33: 0,
            // ch3_nr34: 0,
            // ch3_out_level: 0

            sin_counter: 0.0,
            sample_counter: 0,
            buffer: [0; SOUND_BUFFER_SIZE],
            buffer_idx: 0
        }
    }
}
