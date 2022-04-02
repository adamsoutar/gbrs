use super::apu::APUChannel;
use super::volume_envelope::VolumeEnvelope;
use super::length_function::LengthFunction;

const WAVEFORM_TABLE: [u8; 4] = [
  0b00000001,
  0b00000011,
  0b00001111,
  0b11111100
];

// Doing something every 32k cycles is roughly a 128Hz clock.
const SWEEP_CLOCKS: usize = 32_768;

#[derive(PartialEq)]
enum SweepDirection {
  Up, Down
}

// TODO: Refactor to share code with APUChannel2
//   They are extremely similar to one another minus the sweep register.
pub struct APUChannel1 {
  enabled: bool,
  frequency: usize,
  frequency_timer: usize,
  wave_duty: usize,
  wave_duty_position: usize,
  volume_envelope: VolumeEnvelope,
  length_function: LengthFunction,
  shadow_frequency: usize,
  shadow_frequency_shift: usize,
  sweep_enabled: bool,
  sweep_direction: SweepDirection,
  sweep_period: usize,
  sweep_timer: usize,
  sweep_frame_sequencer: usize
}

impl APUChannel1 {
  pub fn new () -> APUChannel1 {
    APUChannel1 {
      enabled: false,
      frequency: 0,
      frequency_timer: 1,
      wave_duty: 2,
      wave_duty_position: 0,
      volume_envelope: VolumeEnvelope::new(),
      length_function: LengthFunction::new(),
      shadow_frequency: 0,
      shadow_frequency_shift: 0,
      sweep_enabled: false,
      sweep_direction: SweepDirection::Down,
      sweep_period: 0,
      sweep_timer: 1,
      sweep_frame_sequencer: 0
    }
  }

  // This is called when a game writes a 1 in bit 7 of the NR24 register.
  // That means the game is issuing a "restart sound" command
  fn restart_triggered (&mut self) {
    self.volume_envelope.restart_triggered();
    self.length_function.restart_triggered();
    self.length_function.channel_enabled = true;
    self.enabled = true;

    self.shadow_frequency = self.frequency;
    self.sweep_timer = if self.sweep_period == 0 {
      8
    } else {
      self.sweep_period
    };
    if self.sweep_period > 0 || self.shadow_frequency_shift > 0 {
      self.sweep_enabled = true;
    }
    if self.shadow_frequency_shift > 0 {
      self.calculate_sweep_frequency();
    }
    // TODO: Restarting a tone channel resets its frequency_timer to
    //   (2048 - frequency) * 4... I think.
  }

  fn calculate_sweep_frequency (&mut self) -> usize {
    let mut new_frequency = self.shadow_frequency >> self.shadow_frequency_shift;

    if self.sweep_direction == SweepDirection::Down {
      new_frequency = self.shadow_frequency - new_frequency;
    } else {
      new_frequency = self.shadow_frequency + new_frequency;
    }

    if new_frequency > 2047 {
      self.enabled = false;
      // TODO: Do we nee to cap it here? I'm pretty sure this wasn't a 64-bit
      //   value on Gameboy.
    }

    new_frequency
  }

  fn sweep_clock (&mut self) {
    if self.sweep_timer > 0 {
      self.sweep_timer -= 1;

      if self.sweep_timer == 0 {
        self.sweep_timer = if self.sweep_period == 0 { 
          8 
        } else { 
          self.sweep_period 
        };

        if self.sweep_enabled && self.sweep_period != 0 {
          let new_frequency = self.calculate_sweep_frequency();

          if new_frequency <= 2047 && self.shadow_frequency_shift > 0 {
            // println!("Setting a new frequency!");
            self.frequency = new_frequency;
            self.shadow_frequency = new_frequency;

            self.calculate_sweep_frequency();
          }
        }
      }
    }
  }
}

impl APUChannel for APUChannel1 {
  fn step (&mut self) {
    // TODO: I think the Frame Sequencer timers should still be ticking even
    //   if this channel is not enabled. The Frame Sequencer exists outside
    //   the channel.
    if !self.length_function.channel_enabled { return }
    
    self.frequency_timer -= 1;

    if self.frequency_timer == 0 {
      self.frequency_timer = (2048 - self.frequency) * 4;

      // Wrapping pointer into the bits of the WAVEFORM_TABLE value
      self.wave_duty_position += 1;
      if self.wave_duty_position == 8 {
        self.wave_duty_position = 0
      }
    }

    self.sweep_frame_sequencer += 1;
    if self.sweep_frame_sequencer == SWEEP_CLOCKS {
      self.sweep_frame_sequencer = 0;
      self.sweep_clock();
    }

    self.volume_envelope.step();
    self.length_function.step();
  }

  fn read (&self, address: u16) -> u8 {
    match address {
      _ => 0//panic!("Unimplemented APU Channel 2 read {:#06x}", address)
    }
  }

  fn write (&mut self, address: u16, value: u8) {
    match address {
      0xFF10 => {
        // NOTE: This bit is upside down according to Pan Docs. Slightly odd
        //   hardware quirk.
        let sweep_down = (value & 0b0000_1000) > 0;
        self.sweep_direction = if sweep_down { 
          SweepDirection::Down 
        } else { 
          SweepDirection::Up 
        };

        let sweep_shift = (value & 0b0000_0111) as usize;
        self.shadow_frequency_shift = sweep_shift;

        let sweep_period = (value & 0b0111_0000) >> 4;
        self.sweep_period = sweep_period as usize;
      }
      0xFF11 => {
        let wave_duty = (value & 0b1100_0000) >> 6;
        let length = value & 0b0011_1111;
        self.wave_duty = wave_duty as usize;
        // TODO: Is there a way we change this into a generic register_write
        //   function for LengthFunction?
        self.length_function.data = length as usize;
      },
      0xFF12 => self.volume_envelope.register_write(value),
      0xFF13 => {
        // This register sets the bottom 8 bits of the 11-bit
        // frequency register.
        self.frequency = 
          (self.frequency & 0b111_0000_0000)
          | value as usize;
      },
      0xFF14 => {
        // Among other things, this register sets the top 3 bits
        // of the 11-bit frequency register.
        let frequency_bits = value & 0b0000_0111;
        self.frequency =
          (self.frequency & 0b000_1111_1111)
          | ((frequency_bits as usize) << 8);
        
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
    if !self.enabled { return 0. }

    let wave_pattern = WAVEFORM_TABLE[self.wave_duty];
    let amplitude_bit = (wave_pattern & (1 << self.wave_duty_position)) >> self.wave_duty_position;
    
    let dac_input = amplitude_bit as usize * self.volume_envelope.volume;
    // The DAC in the Gameboy outputs between -1.0 and 1.0
    (dac_input as f32 / 7.5) - 1.0
  }
}
