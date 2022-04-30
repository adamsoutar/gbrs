#[cfg(feature = "std")]
use std::{
    io::Read,
    fs::File
};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub struct Rom {
    pub bytes: Vec<u8>
}

impl Rom {
    #[inline(always)]
    pub fn read (&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    #[cfg(feature = "std")]
    pub fn from_file (path: &str) -> Rom {
        let mut buffer = vec![];
        let mut file = File::open(path).expect("Invalid ROM path");
        file.read_to_end(&mut buffer).expect("Unable to read ROM file");

        Rom {
            bytes: buffer
        }
    }

    pub fn from_bytes (bytes: Vec<u8>) -> Rom {
        Rom {
            bytes
        }
    }
}
