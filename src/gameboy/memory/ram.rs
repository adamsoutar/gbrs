pub struct Ram {
    pub bytes: Vec<u8>
}

impl Ram {
    pub fn read (&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    pub fn write (&mut self, address: u16, value: u8) {
        self.bytes[address as usize] = value;
    }

    pub fn new (size: usize) -> Ram {
        Ram {
            bytes: vec![0; size]
        }
    }
}
