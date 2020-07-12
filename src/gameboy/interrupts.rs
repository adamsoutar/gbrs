pub struct InterruptFields {
    pub v_blank: bool,
    pub lcd_stat: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool
}

impl InterruptFields {
    pub fn serialise (&self) -> u8 {
        // TODO
        0
    }

    // TODO: Check if these actuall do all start false
    pub fn new () -> InterruptFields {
        InterruptFields {
            v_blank: false,
            lcd_stat: false,
            timer: false,
            serial: false,
            joypad: false
        }
    }
}

pub struct Interrupts {
    pub enable: InterruptFields,
    pub flag: InterruptFields,

    // "Interrupts master enabled" flag
    pub ime: bool
}

impl Interrupts {
    // TODO: These
    // Called when GB writes to FFFF
    pub fn enable_write (&mut self, value: u8) {

    }

    // Called when GB writes to FF0F
    pub fn flag_write (&mut self, value: u8) {

    }

    // Called when GB reads from FFFF
    pub fn enable_read (&self) -> u8 {
        self.enable.serialise()
    }

    // Called when GB reads from FF0F
    pub fn flag_read (&self) -> u8 {
        self.flag.serialise()
    }

    pub fn new () -> Interrupts {
        Interrupts {
            enable: InterruptFields::new(),
            flag: InterruptFields::new(),

            ime: false
        }
    }
}
