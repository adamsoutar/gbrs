// RAM with a save file
use crate::{callbacks::CALLBACKS, cartridge::Cartridge, memory::ram::Ram};

// The amount of milliseconds we wait before saving our save file
// (otherwise eg. Link's Awakening would write 2,700 save files
//  on its first frame)
const DEBOUNCE_MILLIS: usize = 1000;

pub struct BatteryBackedRam {
    pub ram: Ram,
    pub size: usize,

    cart: Cartridge,

    battery_enabled: bool,
    changed_since_last_save: bool,
    last_saved_at: usize
}

impl BatteryBackedRam {
    pub fn read(&self, address: u16) -> u8 {
        self.ram.read(address)
    }

    pub fn read_usize(&self, address: usize) -> u8 {
        self.ram.bytes[address]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.ram.write(address, value);
        self.changed_since_last_save = true;
    }

    pub fn write_usize(&mut self, address: usize, value: u8) {
        self.ram.bytes[address] = value;
        self.changed_since_last_save = true;
    }

    pub fn step(&mut self, ms_since_boot: usize) {
        if !self.changed_since_last_save || !self.battery_enabled {
            return;
        }

        let millis_since_last_save = ms_since_boot - self.last_saved_at;

        if millis_since_last_save >= DEBOUNCE_MILLIS {
            self.last_saved_at = ms_since_boot;
            self.save_ram_contents()
        }
    }

    fn save_ram_contents(&mut self) {
        self.changed_since_last_save = false;

        (CALLBACKS.lock().save)(
            &self.cart.title[..],
            &self.cart.rom_path[..],
            &self.ram.bytes,
        );
    }

    pub fn new(cart: Cartridge, additional_ram_size: usize, battery_enabled: bool) -> BatteryBackedRam {
        // Some MBCs, like MBC2, always have a few bytes of RAM installed.
        // The cartridge header only tells us about additional external RAM.
        let ram_size = cart.ram_size + additional_ram_size;

        let save_contents = (CALLBACKS.lock().load)(
            &cart.title[..],
            &cart.rom_path[..],
            ram_size,
        );

        let ram = Ram::from_bytes(save_contents, ram_size);

        BatteryBackedRam {
            ram,
            size: ram_size,

            cart,
            battery_enabled,
            changed_since_last_save: false,

            last_saved_at: 0
        }
    }
}
