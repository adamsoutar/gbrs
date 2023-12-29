use crate::cartridge::Cartridge;
use crate::log;
use crate::memory::battery_backed_ram::BatteryBackedRam;
use crate::memory::mbcs::MBC;
use crate::memory::rom::Rom;

// 8KB (one RAM bank) in bytes
pub const KB_8: usize = 8_192;
// 16KB (one ROM bank) in bytes
pub const KB_16: usize = 16_384;

pub struct MBC5 {
    pub rom: Rom,
    pub rom_bank: u16,

    pub ram: BatteryBackedRam,
    pub ram_enabled: bool,
    pub ram_bank: u8,

    has_shown_ram_warning: bool
}

impl MBC for MBC5 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0..=0x3FFF => self.read_bank(0, address),
            0x4000..=0x7FFF => self.read_bank(self.rom_bank, address - 0x4000),
            _ => panic!("Unsupported MBC5 read at {:#06x}", address)
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0A) == 0x0A;
            },
            0x2000..=0x2FFF => {
                // TODO: Are bank numbers masked to the max bank number?

                // No zero check. You can map bank 0 twice on MBC5.
                self.rom_bank =
                    (self.rom_bank & 0b0000_0001_0000_0000) | (value as u16);
            },
            0x3000..=0x3FFF => {
                let bit = if value > 0 { 1 } else { 0 };
                self.rom_bank =
                    (self.rom_bank & 0b0000_0000_1111_1111)
                    | (bit << 8);
            },
            0x4000 ..= 0x5FFF => {
                // TODO: Rumble. If the MBC has rumble circuitry, this may be
                //   wrong because we pass on bit 3.
                if value > 0x0F { return; }
                self.ram_bank = value;
            },
            _ => {}
        }
    }

    fn ram_read(&self, address: u16) -> u8 {
        if !self.ram_enabled && !self.has_shown_ram_warning {
            log!("[WARN] MBC5 RAM read while disabled");
        }

        let banked_address = address as usize + self.ram_bank as usize * KB_8;

        if banked_address >= self.ram.size {
            return 0xFF;
        }

        self.ram.read_usize(banked_address)
    }

    fn ram_write(&mut self, address: u16, value: u8) {
        if !self.ram_enabled && !self.has_shown_ram_warning {
            log!("[WARN] MBC5 RAM write while disabled");
            // Otherwise the game is slowed down by constant debug printing
            self.has_shown_ram_warning = true;
        }

        let banked_address = address as usize + self.ram_bank as usize * KB_8;

        if banked_address >= self.ram.size {
            return;
        }

        self.ram.write_usize(banked_address, value)
    }

    fn step(&mut self, ms_since_boot: usize) {
        self.ram.step(ms_since_boot)
    }
}

impl MBC5 {
    fn read_bank(&self, bank: u16, address: u16) -> u8 {
        let ub = bank as usize;
        let ua = address as usize;
        self.rom.bytes[KB_16 * ub + ua]
    }

    pub fn new(cart_info: Cartridge, rom: Rom) -> Self {
        // TODO: Banked RAM
        if cart_info.ram_size > 8_192 {
            panic!("gbrs doesn't support banked (>=32K) MBC5 RAM");
        }

        let has_battery = cart_info.cart_type == 0x03;
        MBC5 {
            rom,
            rom_bank: 1,
            ram: BatteryBackedRam::new(cart_info, 0, has_battery),
            ram_enabled: false,
            ram_bank: 0,
            has_shown_ram_warning: false
        }
    }
}
