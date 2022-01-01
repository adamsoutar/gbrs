use std::io::Read;
use std::fs::File;

pub struct Ram {
    pub bytes: Vec<u8>,
    pub size: usize
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
            bytes: vec![0; size],
            size
        }
    }

    pub fn from_file (path: &str, expected_size: usize) -> Ram {
        let mut buffer = vec![];
        let mut file = File::open(path).expect("Invalid save file path");
        file.read_to_end(&mut buffer).expect("Unable to read save file");

        if buffer.len() != expected_size {
            panic!("Save file was not the expected length")
        }

        Ram {
            bytes: buffer,
            size: expected_size
        }
    }
}
