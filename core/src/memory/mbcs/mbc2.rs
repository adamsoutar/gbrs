use crate::cartridge::Cartridge;
use crate::log;
use crate::memory::battery_backed_ram::BatteryBackedRam;
use crate::memory::mbcs::MBC;
use crate::memory::rom::Rom;

// 16KB (one bank size) in bytes
pub const KB_16: usize = 16_384;

pub struct MBC2 {
    pub rom: Rom,
    pub rom_bank: u8,

    pub ram: BatteryBackedRam,
    pub ram_enabled: bool,

    has_shown_ram_warning: bool
}

impl MBC for MBC2 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0..=0x3FFF => self.read_bank(0, address),
            0x4000..=0x7FFF => self.read_bank(self.rom_bank, address - 0x4000),
            _ => panic!("Unsupported MBC2 read at {:#06x}", address)
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0A) == 0x0A;
            },
            0x2000..=0x3FFF => {
                let mut n = value & 0b1111;
                if n == 0 {
                    n = 1
                }
                self.rom_bank = n
            },
            _ => {}
        }
    }

    fn ram_read(&self, address: u16) -> u8 {
        if !self.ram_enabled && !self.has_shown_ram_warning {
            log!("[WARN] MBC2 RAM read while disabled");
        }

        // When an address outside of RAM space is read, the gameboy
        // doesn't seem to be intended to crash.
        // Not sure what to return here, but unusable RAM on the GB itself
        // returns 0xFF
        if address as usize >= self.ram.size {
            return 0xFF;
        }

        self.ram.read(address)
    }

    fn ram_write(&mut self, address: u16, value: u8) {
        if !self.ram_enabled && !self.has_shown_ram_warning {
            log!("[WARN] MBC2 RAM write while disabled");
            // Otherwise the game is slowed down by constant debug printing
            self.has_shown_ram_warning = true;
        }

        // See note in ram_read
        if address as usize >= self.ram.size {
            return;
        }

        // NOTE: Only the bottom 4 bits of the written byte are actually
        //   saved on an MBC2, it's half-byte RAM. The top half is
        //   undefined when read.
        self.ram.write(address, value & 0xF)
    }

    fn step(&mut self, ms_since_boot: usize) {
        self.ram.step(ms_since_boot)
    }
}

impl MBC2 {
    fn read_bank(&self, bank: u8, address: u16) -> u8 {
        let ub = bank as usize;
        let ua = address as usize;
        self.rom.bytes[KB_16 * ub + ua]
    }

    pub fn new(cart_info: Cartridge, rom: Rom) -> Self {
        let has_battery = cart_info.cart_type == 0x06;
        MBC2 {
            rom,
            ram_enabled: false,
            rom_bank: 1,
            // The MBC2 always has 512 (half-)bytes of RAM built-in
            ram: BatteryBackedRam::new(cart_info, 512, has_battery),
            has_shown_ram_warning: false
        }
    }
}
