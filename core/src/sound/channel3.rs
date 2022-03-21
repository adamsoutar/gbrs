use crate::constants::*;
use crate::memory::ram::Ram;
use super::apu::APUChannel;
use super::length_function::LengthFunction;

pub struct APUChannel3 {
  frequency: usize,
  frequency_timer: usize,
  // TODO: Is this actually the same bool as the enable within length_function?
  master_enable: bool,
  length_function: LengthFunction,
  wave_ram: Ram,
  wave_ram_ptr: usize,
  volume_shift: u8
}

impl APUChannel3 {
  pub fn new () -> APUChannel3 {
    APUChannel3 {
      frequency: 0,
      frequency_timer: 1,
      master_enable: false,
      length_function: LengthFunction::new(),
      wave_ram: Ram::new(WAVE_RAM_SIZE),
      wave_ram_ptr: 0,
      volume_shift: 0
    }
  }  

  fn restart_triggered (&mut self) {
    self.length_function.restart_triggered();
    self.length_function.channel_enabled = true;
    self.wave_ram_ptr = 0;
  }
}

impl APUChannel for APUChannel3 {
  fn step (&mut self) {
    if !self.length_function.channel_enabled { return }

    self.frequency_timer -= 1;

    if self.frequency_timer == 0 {
      self.frequency_timer = (2048 - self.frequency) * 2;

      // NOTE: This points to a *nibble* of Wave RAM, not a byte.
      self.wave_ram_ptr += 1;
      if self.wave_ram_ptr == 32 {
        self.wave_ram_ptr = 0;
      }
    }

    self.length_function.step();
  }

  fn read (&self, address: u16) -> u8 {
    match address {
      0xFF1A => (self.master_enable as u8) >> 7,
      WAVE_RAM_START ..= WAVE_RAM_END => self.wave_ram.read(address - WAVE_RAM_START),
      _ => 0//panic!("Unimplemented APU Channel 3 read {:#06x}", address)
    }
  }

  fn write (&mut self, address: u16, value: u8) {
    match address {
      0xFF1A => {
        self.master_enable = (value & 0b1000_0000) > 0;
      },
      0xFF1B => {
        self.length_function.data = value as usize;
      },
      0xFF1C => {
        self.volume_shift = (value & 0b0110_0000) >> 5;
      },
      0xFF1D => {
        self.frequency = 
          (self.frequency & 0b111_0000_0000)
          | value as usize;
      },
      0xFF1E => {
        let frequency_bits = value & 0b0000_0111;
        self.frequency =
          (self.frequency & 0b000_1111_1111)
          | ((frequency_bits as usize) << 8);

        self.length_function.timer_enabled = (value & 0b0100_0000) > 0;

        if (value & 0b1000_0000) > 0 {
          self.restart_triggered();
        }
      },
      WAVE_RAM_START ..= WAVE_RAM_END => self.wave_ram.write(address - WAVE_RAM_START, value),
      _ => unreachable!()
    }
  }

  fn sample (&self) -> f32 {
    if !self.length_function.channel_enabled { return 0. }

    // This implementation is a bit guessed for now :)
    // Documentation on Channel 3 seems a little bit thin
    let wave_byte = self.wave_ram.bytes[self.wave_ram_ptr / 2];
    let mut wave_nibble = if self.wave_ram_ptr % 2 == 0 {
      wave_byte & 0x0F
    } else {
      wave_byte >> 4
    };

    wave_nibble = wave_nibble >> match self.volume_shift {
      0 => 4,
      1 => 0,
      2 => 1,
      3 => 2,
      _ => unreachable!()
    };

    (wave_nibble as f32 / 7.5) - 1.0
  }
}
