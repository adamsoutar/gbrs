#[derive(PartialEq)]
enum EnvelopeDirection {
  Up, Down
}

// Doing something every 65k cycles is roughly a 64Hz clock.
const ENV_CLOCKS: usize = 65_536;

pub struct VolumeEnvelope {
  initial_volume: usize,
  direction: EnvelopeDirection,
  sweep_period: usize,
  period_timer: usize,
  volume_timer: usize,
  pub volume: usize,
}

impl VolumeEnvelope {
  pub fn step (&mut self) {
    self.volume_timer += 1;
    if self.volume_timer == ENV_CLOCKS {
      self.volume_timer = 0;
      self.clock();
    }
  }

  pub fn restart_triggered (&mut self) {
    self.period_timer = self.sweep_period;
    self.volume = self.initial_volume;
  }
  
  pub fn register_write (&mut self, value: u8) {
    self.initial_volume = (value as usize & 0b1111_0000) >> 4;
    self.direction = if (value & 0b0000_1000) > 0 { 
      EnvelopeDirection::Up 
    } else {
      EnvelopeDirection::Down
    };
    self.sweep_period = value as usize & 0b0000_0111;
  }

  // Called at 64Hz
  fn clock (&mut self) {
    if self.sweep_period == 0 { return }

    if self.period_timer > 0 {
      self.period_timer -= 1;
    }

    if self.period_timer == 0 {
      self.period_timer = self.sweep_period;

      if self.direction == EnvelopeDirection::Up && self.volume < 0xF {
        self.volume += 1;
      }
      if self.direction == EnvelopeDirection::Down && self.volume > 0 {
        self.volume -= 1;
      }
    }
  }

  pub fn new () -> VolumeEnvelope {
    VolumeEnvelope {
      initial_volume: 0,
      direction: EnvelopeDirection::Down,
      sweep_period: 0,
      period_timer: 0,
      volume: 0,
      volume_timer: 0
    }
  }
}
