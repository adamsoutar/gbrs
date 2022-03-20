use crate::log;
use crate::sound::apu::APUChannel;

const WAVEFORM_TABLE: [u8; 4] = [
  0b00000001,
  0b00000011,
  0b00001111,
  0b11111100
];
// Doing something every 65k cycles is roughly a 64Hz clock.
const ENV_VOLUME_CLOCKS: usize = 65_536;
// 256Hz
const LENGTH_CLOCKS: usize = 16_392;

#[derive(PartialEq)]
enum EnvelopeDirection {
  Up, Down
}

pub struct APUChannel2 {
  enabled: bool,
  frequency: usize,
  frequency_timer: usize,
  wave_duty: usize,
  wave_duty_position: usize,
  // Envelope stuff
  env_initial_volume: usize,
  env_direction: EnvelopeDirection,
  env_sweep_period: usize,
  env_period_timer: usize,
  env_volume: usize,
  env_volume_timer: usize,
  length_timer: usize,
  length_data: usize,
  length_timer_enabled: bool,
  length_clock_timer: usize
}

impl APUChannel2 {
  pub fn new () -> APUChannel2 {
    APUChannel2 {
      enabled: false,
      frequency: 0,
      frequency_timer: 1,
      wave_duty: 2,
      wave_duty_position: 0,
      env_initial_volume: 0,
      env_direction: EnvelopeDirection::Down,
      env_sweep_period: 0,
      env_period_timer: 0,
      env_volume: 0,
      env_volume_timer: 0,
      length_timer: 0,
      length_data: 0,
      length_timer_enabled: false,
      length_clock_timer: 0
    }
  }

  // This is called when a game writes a 1 in bit 7 of the NR24 register.
  // That means the game is issuing a "restart sound" command
  fn restart_triggered (&mut self) {
    self.env_period_timer = self.env_sweep_period;
    self.env_volume = self.env_initial_volume;
    self.length_timer = 64 - self.length_data;
    self.enabled = true;
  }

  // Called at 64Hz
  fn envelope_clock (&mut self) {
    if self.env_sweep_period == 0 { return }

    if self.env_period_timer > 0 {
      self.env_period_timer -= 1;
    }

    if self.env_period_timer == 0 {
      self.env_period_timer = self.env_sweep_period;

      if self.env_direction == EnvelopeDirection::Up && self.env_volume < 0xF {
        self.env_volume += 1;
      }
      if self.env_direction == EnvelopeDirection::Down && self.env_volume > 0 {
        self.env_volume -= 1;
      }
    }
  }

  // Called at 256Hz
  fn length_clock (&mut self) {
    if self.length_timer > 0 {
      self.length_timer -= 1;
    }

    if self.length_timer == 0 {
      if self.length_timer_enabled {
        self.length_timer = 64 - self.length_data;
        self.enabled = false;
      }
    }
  }
}

impl APUChannel for APUChannel2 {
  fn step (&mut self) {
    if !self.enabled { return }
    
    self.frequency_timer -= 1;

    if self.frequency_timer == 0 {
      self.frequency_timer = (2048 - self.frequency) * 4;

      // Wrapping pointer into the bits of the WAVEFORM_TABLE value
      self.wave_duty_position += 1;
      if self.wave_duty_position == 8 {
        self.wave_duty_position = 0
      }
    }

    self.env_volume_timer += 1;
    if self.env_volume_timer == ENV_VOLUME_CLOCKS {
      self.env_volume_timer = 0;
      self.envelope_clock();
    }

    self.length_clock_timer += 1;
    if self.length_clock_timer == LENGTH_CLOCKS {
      self.length_clock_timer = 0;
      self.length_clock();
    }
  }

  fn read (&self, address: u16) -> u8 {
    match address {
      _ => 0//panic!("Unimplemented APU Channel 2 read {:#06x}", address)
    }
  }

  fn write (&mut self, address: u16, value: u8) {
    match address {
      0xFF16 => {
        let wave_duty = (value & 0b1100_0000) >> 6;
        let length = value & 0b0011_1111;
        self.wave_duty = wave_duty as usize;
        self.length_data = length as usize;
      },
      0xFF17 => {
        self.env_initial_volume = (value as usize & 0b1111_0000) >> 4;
        self.env_direction = if (value & 0b0000_1000) > 0 { 
          EnvelopeDirection::Up 
        } else {
          EnvelopeDirection::Down
        };
        self.env_sweep_period = value as usize & 0b0000_0111;
      },
      0xFF18 => {
        // This register sets the bottom 8 bits of the 11-bit
        // frequency register.
        self.frequency = 
          (self.frequency & 0b111_0000_0000)
          | value as usize;
      },
      0xFF19 => {
        // Among other things, this register sets the top 3 bits
        // of the 11-bit frequency register.
        let frequency_bits = 0b0000_0111;
        self.frequency =
          (self.frequency & 0b000_1111_1111)
          | ((frequency_bits as usize) << 8);
        
        self.length_timer = 0;
        self.length_clock_timer = 0;
        self.length_timer_enabled = (value & 0b0100_0000) > 0;

        if (value & 0b1000_0000) > 0 {
          self.restart_triggered();
        }
      },
      _ => unreachable!()
    }
  }

  fn sample (&self) -> i16 {
    if !self.enabled { return 0 }

    let wave_pattern = WAVEFORM_TABLE[self.wave_duty];
    let amplitude_bit = (wave_pattern & (1 << self.wave_duty_position)) >> self.wave_duty_position;
    
    let dac_input = amplitude_bit as usize * self.env_volume;
    // The DAC in the Gameboy outputs between -1.0 and 1.0
    let dac_output = (dac_input as f64 / 7.5) - 1.0;

    (dac_output * 7500.0) as i16
  }
}
