use crate::gameboy::memory::mbcs::MBC;
use crate::gameboy::memory::rom::Rom;
use crate::gameboy::memory::ram::Ram;
use crate::gameboy::cartridge::Cartridge;

// 16KB (one bank size) in bytes
pub const KB_16: usize = 16_384;

pub struct MBC1 {
    pub rom: Rom,
    pub rom_bank: u8,

    // Not really supported yet
    pub ram: Ram,
    pub ram_enabled: bool,

    pub battery: bool
}

impl MBC for MBC1 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0 ..= 0x3FFF => self.read_bank(0, address),
            0x4000 ..= 0x7FFF => self.read_bank(self.rom_bank, address - 0x4000),
            _ => panic!("Unsupported MBC1 read at {:#06x}", address)
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000 ..= 0x1FFF => {
                self.ram_enabled = (value & 0x0A) == 0x0A;
            },
            0x2000 ..= 0x3FFF => {
                // TODO: More advanced banking support
                let mut n = value & 0b11111;
                if n == 0 { n = 1 }
                self.rom_bank = n
            },
            _ => println!("Unsupported MBC1 write at {:#06x} (value: {:#04x})", address, value)
        }
    }

    fn ram_read(&self, _: u16) -> u8 {
        // TODO
        0
    }

    fn ram_write(&mut self, _: u16, _: u8) {
        // TODO
    }
}

impl MBC1 {
    fn read_bank(&self, bank: u8, address: u16) -> u8 {
        let ub = bank as usize;
        let ua = address as usize;
        self.rom.bytes[KB_16 * ub + ua]
    }

    pub fn new (cart_info: Cartridge, rom: Rom) -> Self {
        // TODO: Support the battery
        // TODO: Support the RAM
        MBC1 {
            rom,
            ram_enabled: false,
            rom_bank: 1,
            ram: Ram::new(cart_info.ram_size),
            battery: cart_info.cart_type == 0x03
        }
    }
}
