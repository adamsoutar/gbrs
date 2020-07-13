use std::io::Read;
use std::fs::File;
use crate::gameboy::cartridge::Cartridge;

pub struct Rom {
    pub bytes: Vec<u8>,
    pub cart_info: Cartridge
}

impl Rom {
    pub fn read (&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    pub fn from_file (path: String) -> Rom {
        let mut buffer = vec![];
        let mut file = File::open(path).expect("Invalid ROM path");
        file.read_to_end(&mut buffer).expect("Unable to read ROM file");

        let cart = Cartridge::parse(&buffer);

        Rom {
            bytes: buffer,
            cart_info: cart
        }
    }
}
