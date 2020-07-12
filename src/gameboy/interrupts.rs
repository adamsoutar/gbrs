pub struct InterruptFields {
    pub v_blank: bool,
    pub lcd_stat: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool
}

// TODO: Impl for From<u8> instead of deserialise()
impl InterruptFields {
    // TODO: Check these go in this order and not backwards
    pub fn serialise (&self) -> u8 {
        let b1 = self.v_blank as u8;
        let b2 = (self.lcd_stat as u8) << 1;
        let b3 = (self.timer as u8) << 2;
        let b4 = (self.serial as u8) << 3;
        let b5 = (self.joypad as u8) << 4;
        b1 | b2 | b3 | b4 | b5
    }

    pub fn deserialise(&mut self, n: u8) {
        self.v_blank = (n & 1) == 1;
        self.lcd_stat = ((n >> 1) & 1) == 1;
        self.timer = ((n >> 2) & 1) == 1;
        self.serial = ((n >> 3) & 1) == 1;
        self.joypad = ((n >> 4) & 1) == 1;
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
        self.enable.deserialise(value)
    }

    // Called when GB writes to FF0F
    pub fn flag_write (&mut self, value: u8) {
        self.flag.deserialise(value)
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
