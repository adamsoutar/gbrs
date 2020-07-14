// TODO: Raise the Joypad interrupt
pub struct Joypad {
    pub directions: bool,

    // The GUI writes these values directly via the keyboard
    // Every frame.
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub a_pressed: bool,
    pub b_pressed: bool,
    pub start_pressed: bool,
    pub select_pressed: bool,
}

fn is_directions (n: u8) -> bool {
    return (n & 0b00010000) == 0;
}

impl Joypad {
    pub fn write (&mut self, n: u8) {
        self.directions = is_directions(n);
    }

    fn direction_bits (&self) -> u8 {
        (!self.right_pressed as u8) |
        ((!self.left_pressed as u8) << 1) |
        ((!self.up_pressed as u8) << 2) |
        ((!self.down_pressed as u8) << 3)
    }

    fn button_bits (&self) -> u8 {
        (!self.a_pressed as u8) |
        ((!self.b_pressed as u8) << 1) |
        ((!self.select_pressed as u8) << 2) |
        ((!self.start_pressed as u8) << 3)
    }

    fn selection_bits (&self) -> u8 {
        if self.directions {
            0b0001_0000
        } else {
            0b0010_0000
        }
    }

    pub fn read (&self) -> u8 {
        let n = if self.directions {
            self.direction_bits()
        } else { self.button_bits() };

        n | self.selection_bits()
    }

    pub fn new () -> Joypad {
        Joypad {
            directions: false,
            up_pressed: false,
            down_pressed: false,
            left_pressed: false,
            right_pressed: false,
            a_pressed: false,
            b_pressed: false,
            start_pressed: false,
            select_pressed: false
        }
    }
}
