#[cfg(feature = "std")]
use std::{fs::File, io::Read};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

#[derive(Clone)]
pub struct Rom {
    pub bytes: Vec<u8>,
    pub path: String,
}

impl Rom {
    #[inline(always)]
    pub fn read(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    #[cfg(feature = "std")]
    pub fn from_file(path: &str) -> Rom {
        let mut buffer = vec![];
        let mut file = File::open(path).expect("Invalid ROM path");
        file.read_to_end(&mut buffer)
            .expect("Unable to read ROM file");

        Rom {
            bytes: buffer,
            path: path.to_string(),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Rom {
        Rom {
            bytes,
            path: String::new(),
        }
    }
}
