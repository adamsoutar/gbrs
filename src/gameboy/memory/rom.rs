use std::io::Read;
use std::fs::File;

pub struct Rom {
    pub bytes: Vec<u8>
}

impl Rom {
    pub fn read (&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    pub fn from_file (path: String) -> Rom {
        let mut buffer = vec![];
        let mut file = File::open(path).expect("Invalid ROM path");
        file.read_to_end(&mut buffer).expect("Unable to read ROM file");

        Rom {
            bytes: buffer
        }
    }
}
