use crate::gameboy::memory::mbcs::MBC;
use crate::gameboy::memory::rom::Rom;

pub struct MBCNone {
    pub rom: Rom
}

impl MBC for MBCNone {
    fn read(&self, address: u16) -> u8 {
        self.rom.read(address)
    }

    fn write(&mut self, _: u16, _: u8) {
        // No MBC ignores writes
    }
}

impl MBCNone {
    pub fn new(rom: Rom) -> Self {
        MBCNone {
            rom
        }
    }
}
