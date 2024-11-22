use crate::cartridge::Cartridge;
use crate::log;
use crate::memory::battery_backed_ram::BatteryBackedRam;
use crate::memory::mbcs::MBC;
use crate::memory::rom::Rom;

// 8KB (one RAM bank size) in bytes
pub const KB_8: usize = 8_192;
// 16KB (one ROM bank size) in bytes
pub const KB_16: usize = KB_8 * 2;

pub struct MBC3 {
    pub rom: Rom,
    pub rom_bank: u8,

    pub ram: BatteryBackedRam,
    pub ram_bank: u8,
    pub ram_enabled: bool,

    // Unique MBC3 feature, sometimes the RAM addresses can be set up to
    // read a Real Time Clock
    pub rtc_select: bool,

    has_shown_ram_warning: bool,
}

impl MBC for MBC3 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0..=0x3FFF => self.read_bank(0, address),
            0x4000..=0x7FFF => self.read_bank(self.rom_bank, address - 0x4000),
            _ => panic!("Unsupported MBC3 read at {:#06x}", address),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0A) == 0x0A;
            },
            0x2000..=0x3FFF => {
                let mut n = value & 0b01111111;
                let max_bank = (self.rom.bytes.len() / KB_16) as u8;
                let bitmask = max_bank - 1;
                n = n & bitmask;
                if n == 0 {
                    n = 1
                }
                // log!("Selecting ROM bank {}", n);
                self.rom_bank = n
            },
            0x4000..=0x5FFF => {
                match value {
                    0x00..=0x03 => {
                        // log!("Selecting RAM bank {}", value);
                        self.ram_bank = value;
                        self.rtc_select = false;
                    },
                    // TODO: This maps Real Time Clock stuff
                    0x08..=0x0C => {
                        self.rtc_select = true;
                    },
                    // This is a noop
                    _ => {},
                }
            },
            0x6000..=0x7FFF => {
                // TODO: RTC latching
            },
            _ => {},
        }
    }

    fn ram_read(&self, address: u16) -> u8 {
        if !self.ram_enabled && !self.has_shown_ram_warning {
            // log!("[WARN] MBC3 RAM read while disabled");
        }

        if self.rtc_select {
            // The game has opted to replace RAM with the value of the RTC.
            // TODO: Properly emulate the Real Time Clock
            // log!("Reading the RTC");
            return 0;
        }

        self.read_ram_bank(self.ram_bank, address)
    }

    fn ram_write(&mut self, address: u16, value: u8) {
        if !self.ram_enabled && !self.has_shown_ram_warning {
            log!("[WARN] MBC3 RAM write while disabled");
            // Otherwise the game is slowed down by constant debug printing
            self.has_shown_ram_warning = true;
        }

        self.write_ram_bank(self.ram_bank, address, value);
    }

    fn step(&mut self, ms_since_boot: usize) {
        self.ram.step(ms_since_boot)
    }
}

impl MBC3 {
    fn read_bank(&self, bank: u8, address: u16) -> u8 {
        let ub = bank as usize;
        let ua = address as usize;
        let final_addr = KB_16 * ub + ua;

        // if final_addr >= self.rom.bytes.len() {
        //     return 0xFF;
        // }

        self.rom.bytes[final_addr]
    }

    fn read_ram_bank(&self, bank: u8, address: u16) -> u8 {
        let ub = bank as usize;
        let ua = address as usize;
        let final_addr = KB_8 * ub + ua;

        // if final_addr >= self.ram.size {
        //     return 0xFF;
        // }

        self.ram.ram.bytes[final_addr]
    }

    fn write_ram_bank(&mut self, bank: u8, address: u16, value: u8) {
        let ub = bank as usize;
        let ua = address as usize;
        let final_addr = KB_8 * ub + ua;

        if final_addr >= self.ram.size {
            return;
        }

        self.ram.write(final_addr as u16, value);
    }

    pub fn new(cart_info: Cartridge, rom: Rom) -> Self {
        let has_battery = cart_info.cart_type == 0x13;
        MBC3 {
            rom,
            rom_bank: 1,
            ram: BatteryBackedRam::new(cart_info, 0, has_battery),
            ram_bank: 0,
            ram_enabled: false,
            rtc_select: false,
            has_shown_ram_warning: false,
        }
    }
}
