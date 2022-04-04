use crate::callbacks::CALLBACKS;
use crate::constants::*;
use super::channel1::APUChannel1;
use super::channel2::APUChannel2;
use super::channel3::APUChannel3;
use super::channel4::APUChannel4;
use super::registers::*;

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

    pub stereo_panning: StereoPanning,

    pub sound_on_register: u8,

    pub channel1: APUChannel1,
    pub channel2: APUChannel2,
    pub channel3: APUChannel3,
    pub channel4: APUChannel4,

    pub sample_counter: usize,
    // This could be a Vec that we check len() against, but we can save the 
    // allocation because we know the size it's always going to be.
    pub buffer: [i16; SOUND_BUFFER_SIZE],
    pub buffer_idx: usize,
    pub buffer_full: bool
}

impl APU {
    pub fn step (&mut self) {
        self.channel1.step();
        self.channel2.step();
        self.channel3.step();
        self.channel4.step();

        self.sample_counter += 1;

        if self.sample_counter == APU_SAMPLE_CLOCKS {
            self.sample_counter = 0;
            self.sample();
        }
    }

    pub fn sample (&mut self) {
        let mut left_sample = 0.;
        let mut right_sample = 0.;

        // TODO: Maybe we could generate these with a macro?
        let chan1 = self.channel1.sample();
        if self.stereo_panning.channel1_left {
            left_sample += chan1;
        }
        if self.stereo_panning.channel1_right {
            right_sample += chan1;
        }

        let chan2 = self.channel2.sample();
        if self.stereo_panning.channel2_left {
            left_sample += chan2;
        }
        if self.stereo_panning.channel2_right {
            right_sample += chan2;
        }

        let chan3 = self.channel3.sample();
        if self.stereo_panning.channel3_left {
            left_sample += chan3;
        }
        if self.stereo_panning.channel3_right {
            right_sample += chan3;
        }

        let chan4 = self.channel4.sample();
        if self.stereo_panning.channel4_left {
            left_sample += chan4;
        }
        if self.stereo_panning.channel4_right {
            right_sample += chan4;
        }

        // Average the 4 channels
        left_sample /= 4.;
        right_sample /= 4.;

        // Adjust for soft-panning
        left_sample *= self.stereo_left_volume;
        right_sample *= self.stereo_right_volume;

        let left_sample_int = (left_sample * 30_000.) as i16;
        let right_sample_int = (right_sample * 30_000.) as i16;

        self.buffer[self.buffer_idx] = left_sample_int;
        self.buffer_idx += 1;
        self.buffer[self.buffer_idx] = right_sample_int;
        self.buffer_idx += 1;

        if self.buffer_idx == SOUND_BUFFER_SIZE {
            self.buffer_idx = 0;
            self.buffer_full = true;
            // unsafe {
            //     (CALLBACKS.play_sound)(&self.buffer)
            // }  
        }
    }

    pub fn read (&self, address: u16) -> u8 {
        match address {
            0xFF24 => self.serialise_nr50(),
            0xFF25 => u8::from(self.stereo_panning.clone()),
            0xFF26 => self.sound_on_register,

            0xFF10..=0xFF14 => self.channel1.read(address),
            0xFF16..=0xFF19 => self.channel2.read(address),
            0xFF1A..=0xFF1E => self.channel3.read(address),
            0xFF20..=0xFF23 => self.channel4.read(address),

            WAVE_RAM_START..=WAVE_RAM_END => self.channel3.read(address),
            _ => 0 //panic!("Unknown read {:#06x} in APU", address)
        }
    }

    pub fn write (&mut self, address: u16, value: u8) {
        match address {
            0xFF24 => self.deserialise_nr50(value),
            0xFF25 => self.stereo_panning = StereoPanning::from(value),
            0xFF26 => self.sound_on_register = value,

            0xFF10..=0xFF14 => self.channel1.write(address, value),
            0xFF16..=0xFF19 => self.channel2.write(address, value),
            0xFF1A..=0xFF1E => self.channel3.write(address, value),
            0xFF20..=0xFF23 => self.channel4.write(address, value),

            WAVE_RAM_START..=WAVE_RAM_END => self.channel3.write(address, value),
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
            stereo_panning: StereoPanning::from(0),
            sound_on_register: 0,

            channel1: APUChannel1::new(),
            channel2: APUChannel2::new(),
            channel3: APUChannel3::new(),
            channel4: APUChannel4::new(),

            sample_counter: 0,
            buffer: [0; SOUND_BUFFER_SIZE],
            buffer_idx: 0,
            buffer_full: false
        }
    }
}
