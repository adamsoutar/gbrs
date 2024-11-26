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
    pub fn step(&mut self) {
        self.clock_timer += 1;
        if self.clock_timer == LENGTH_CLOCKS {
            self.clock_timer = 0;
            self.clock();
        }
    }

    pub fn restart_triggered(&mut self) {
        // TODO: This behaviour isn't quite right
        //   https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware#Trigger_Event
        self.timer = 64;
        self.timer = self.timer.saturating_sub(self.data);
    }

    // Called at 256Hz
    fn clock(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        }

        if self.timer == 0 {
            if self.timer_enabled {
                // TODO: Without saturating_sub, this causes panics after moving to
                //   48KHz audio when flapping with the rabbit ears in Mario Land 2
                self.timer = 64;
                self.timer = self.timer.saturating_sub(self.data);
                self.channel_enabled = false;
            }
        }
    }

    pub fn new() -> LengthFunction {
        LengthFunction {
            channel_enabled: true,
            timer: 0,
            data: 0,
            timer_enabled: false,
            clock_timer: 0,
        }
    }
}
