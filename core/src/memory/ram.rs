#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

pub struct Ram {
    pub bytes: Vec<u8>,
    pub size: usize,
}

impl Ram {
    #[inline(always)]
    pub fn read(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    #[inline(always)]
    pub fn write(&mut self, address: u16, value: u8) {
        self.bytes[address as usize] = value;
    }

    pub fn new(size: usize) -> Ram {
        Ram::with_filled_value(size, 0)
    }

    pub fn with_filled_value(size: usize, default_value: u8) -> Ram {
        Ram {
            bytes: vec![default_value; size],
            size,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, expected_size: usize) -> Ram {
        if bytes.len() != expected_size {
            panic!("Save file was not the expected length")
        }

        Ram {
            bytes,
            size: expected_size,
        }
    }
}
