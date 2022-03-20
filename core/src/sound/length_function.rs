// 256Hz
const LENGTH_CLOCKS: usize = 16_392;

pub struct LengthFunction {
  pub channel_enabled: bool,
  pub data: usize,
  pub timer_enabled: bool,
  timer: usize,
  clock_timer: usize,
}

impl LengthFunction {
  pub fn step (&mut self) {
    self.clock_timer += 1;
    if self.clock_timer == LENGTH_CLOCKS {
      self.clock_timer = 0;
      self.clock();
    }
  }

  pub fn restart_triggered (&mut self) {
    self.timer = 64 - self.data;
  }

  // Called at 256Hz
  fn clock (&mut self) {
    if self.timer > 0 {
      self.timer -= 1;
    }

    if self.timer == 0 {
      if self.timer_enabled {
        self.timer = 64 - self.data;
        self.channel_enabled = false;
      }
    }
  }

  pub fn new () -> LengthFunction {
    LengthFunction {
      channel_enabled: false,
      timer: 0,
      data: 0,
      timer_enabled: false,
      clock_timer: 0,
    }
  }
}
