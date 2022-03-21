use super::apu::APUChannel;
use super::length_function::LengthFunction;
use super::volume_envelope::VolumeEnvelope;

pub struct APUChannel4 {
  // TODO: Size these better. Maybe u32 rather than usize?
  //   Not super important at all, but just to be sure.
  frequency_timer: usize,
  length_function: LengthFunction,
  volume_envelope: VolumeEnvelope,
  // Linear Feedback Shift Register
  // NOTE: The LFSR is 15-bits wide on the Gameboy. We'll use a 16-bit type
  //   to represent it.
  lfsr: u16,
  // TODO: Is this a u8? Does shifting large-ish numbers overflow?
  divisor_shift: usize,
  half_width_mode: bool,
  divisor_code: usize
}

impl APUChannel4 {
  pub fn new () -> APUChannel4 {
    APUChannel4 {
      frequency_timer: 1,
      length_function: LengthFunction::new(),
      volume_envelope: VolumeEnvelope::new(),
      lfsr: 0,
      divisor_shift: 0,
      half_width_mode: false,
      divisor_code: 0
    }
  }

  fn restart_triggered (&mut self) {
    self.length_function.restart_triggered();
    self.volume_envelope.restart_triggered();
    self.lfsr = 0b0111_1111_1111_1111;
  }

  fn get_divisor (&self) -> usize {
    match self.divisor_code {
      0 => 8,
      1 => 16,
      2 => 32,
      3 => 48,
      4 => 64,
      5 => 80,
      6 => 96,
      7 => 112,
      _ => unreachable!()
    }
  }
}

impl APUChannel for APUChannel4 {
    fn step (&mut self) {
      if !self.length_function.channel_enabled { return }

      self.frequency_timer -= 1;

      if self.frequency_timer == 0 {
        self.frequency_timer = self.get_divisor() << self.divisor_shift;

        // Pseudo-random white noise generation
        let xor = (self.lfsr & 0b01) ^ ((self.lfsr & 0b10) >> 1);
        self.lfsr = (self.lfsr >> 1) | (xor << 14);

        if self.half_width_mode {
          self.lfsr = (self.lfsr & 0b0011_1111) | (xor << 6);
        }
      }
      
      self.volume_envelope.step();
      self.length_function.step();
    }

    fn read (&self, address: u16) -> u8 {
      match address {
        _ => 0//panic!("Unimplemented APU Channel 4 read {:#06x}", address)
      }
    }

    fn write (&mut self, address: u16, value: u8) {
      match address {
        0xFF20 => {
          let length = value & 0b0011_1111;
          self.length_function.data = length as usize;
        },
        0xFF21 => self.volume_envelope.register_write(value),
        0xFF22 => {
          // Polynomial register
          self.divisor_shift = ((value & 0b1111_0000) >> 4) as usize;
          self.half_width_mode = (value & 0b0000_1000) > 0;
          self.divisor_code = (value & 0b0000_0111) as usize;
        },
        0xFF23 => {
          self.length_function.timer_enabled = (value & 0b0100_0000) > 0;

          if (value & 0b1000_0000) > 0 {
            self.restart_triggered();
          }
        },
        _ => unreachable!()
      }
    }

    fn sample (&self) -> f32 {
      if !self.length_function.channel_enabled { return 0. }

      let lfsr_bit = !(self.lfsr) & 1;

      let dac_input = lfsr_bit as usize * self.volume_envelope.volume;
      (dac_input as f32 / 7.5) - 1.0
    }
}
