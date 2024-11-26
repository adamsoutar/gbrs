use crate::log;

pub struct CgbSpeedSwitch {
    pub armed: bool,
    pub current_speed_is_double: bool,
    cgb_features: bool,
}

// TODO: Actually act on this for CPU speed.
//   This just tracks the byte state.
impl CgbSpeedSwitch {
    pub fn write_switch_byte(&mut self, value: u8) {
        if self.cgb_features {
            self.armed = value & 1 > 0;
        }
    }
    pub fn read_switch_byte(&self) -> u8 {
        let top_bit = match self.current_speed_is_double {
            true => 0x80,
            false => 0x00,
        };
        let bottom_bit = match self.armed {
            true => 0x01,
            false => 0x00,
        };
        top_bit | bottom_bit
    }
    pub fn execute_speed_switch(&mut self) {
        self.armed = false;
        self.current_speed_is_double = !self.current_speed_is_double;
        log!(
            "Performing CGB speed switch. New speed: {}",
            match self.current_speed_is_double {
                true => "Double",
                false => "Single",
            }
        );
    }

    pub fn new(cgb_features: bool) -> Self {
        CgbSpeedSwitch {
            armed: false,
            current_speed_is_double: false,
            cgb_features,
        }
    }
}
