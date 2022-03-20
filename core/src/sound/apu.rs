use crate::callbacks::CALLBACKS;
use crate::constants::*;
use crate::memory::ram::Ram;
use crate::sound::channel2::APUChannel2;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub trait APUChannel {
    fn step(&mut self);
    fn sample(&self) -> f32;
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

// Audio processing unit
// NOTE: Max APU frequency seems to be 131072 Hz
pub struct APU {
    pub stereo_left_volume: f32,
    pub stereo_right_volume: f32,

    pub stereo_channel_control: u8,

    pub sound_on_register: u8,

    pub wave_ram: Ram,

    pub channel2: APUChannel2,

    pub sample_counter: usize,
    // This could be a Vec that we check len() against, but we can save the 
    // allocation because we know the size it's always going to be.
    pub buffer: [i16; SOUND_BUFFER_SIZE],
    pub buffer_idx: usize
}

impl APU {
    pub fn step (&mut self) {
        self.channel2.step();

        self.sample_counter += 1;

        if self.sample_counter == APU_SAMPLE_CLOCKS {
            self.sample_counter = 0;
            self.sample();
        }
    }

    pub fn sample (&mut self) {
        let mut mixed_sample: f32 = 0.;
        mixed_sample += self.channel2.sample();

        // Average the 4 channels
        mixed_sample /= 4.;
        
        // TODO: Audio panning.
        //   Right now we essentially play mono down two channels.
        let i16_sample = (mixed_sample * 30_000.) as i16;
        let left_sample = i16_sample;
        let right_sample = i16_sample;

        self.buffer[self.buffer_idx] = left_sample;
        self.buffer_idx += 1;
        self.buffer[self.buffer_idx] = right_sample;
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

            0xFF16..=0xFF19 => self.channel2.read(address),

            WAVE_RAM_START ..= WAVE_RAM_END => self.wave_ram.read(address - WAVE_RAM_START),
            _ => 0 //panic!("Unknown read {:#06x} in APU", address)
        }
    }

    pub fn write (&mut self, address: u16, value: u8) {
        match address {
            0xFF24 => self.deserialise_nr50(value),
            0xFF25 => self.stereo_channel_control = value,
            0xFF26 => self.sound_on_register = value,

            0xFF16..=0xFF19 => self.channel2.write(address, value),

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

            channel2: APUChannel2::new(),

            sample_counter: 0,
            buffer: [0; SOUND_BUFFER_SIZE],
            buffer_idx: 0
        }
    }
}
