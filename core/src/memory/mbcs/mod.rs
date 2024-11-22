use crate::cartridge::Cartridge;
use crate::log;
use crate::memory::rom::Rom;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

pub trait MBC {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    fn ram_read(&self, address: u16) -> u8;
    fn ram_write(&mut self, address: u16, value: u8);

    // Mostly used to debounce battery-backed RAM saves
    fn step(&mut self, ms_since_boot: usize);
}

mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod none;

pub fn mbc_from_info(cart_info: Cartridge, rom: Rom) -> Box<dyn MBC> {
    log!("Loading game \"{}\"", cart_info.title);
    log!("Extra chips: {}", get_cart_type_string(&cart_info));
    log!("ROM size: {}KB", cart_info.rom_size / 1024);
    log!("RAM size: {}KB", cart_info.ram_size / 1024);

    match cart_info.cart_type {
        0x00 => Box::new(none::MBCNone::new(rom)),
        0x01 ..= 0x03 => Box::new(mbc1::MBC1::new(cart_info, rom)),
        0x05 ..= 0x06 => Box::new(mbc2::MBC2::new(cart_info, rom)),
        0x0F ..= 0x13 => Box::new(mbc3::MBC3::new(cart_info, rom)),
        0x19 ..= 0x1E => Box::new(mbc5::MBC5::new(cart_info, rom)),
        _ => panic!("gbrs doesn't support this cartridge's memory controller ({:#04x}).", cart_info.cart_type)
    }
}

fn get_cart_type_string(cart_info: &Cartridge) -> &str {
    match cart_info.cart_type {
        0x00 => "None",
        0x01 => "MBC1",
        0x02 => "MBC1 + RAM",
        0x03 => "MBC1 + RAM + BATTERY",
        // There are some gaps where Pan Docs doesn't define what they are
        0x05 => "MBC2",
        0x06 => "MBC2 + BATTERY",

        0x08 => "ROM + RAM (Unofficial)", // No gameboy game uses these
        0x09 => "ROM + RAM + BATTERY (Unofficial)",

        0x0B => "MMM01",
        0x0C => "MMM01 + RAM",
        0x0D => "MMM01 + RAM + BATTERY",

        0x0F => "MBC3 + TIMER + BATTERY",
        0x10 => "MBC3 + TIMER + RAM + BATTERY",
        0x11 => "MBC3",
        0x12 => "MBC3 + RAM",
        0x13 => "MBC3 + RAM + BATTERY",

        // There is no MBC4. There is superstition about the number 4 in Japan.

        0x19 => "MBC5",
        0x1A => "MBC5 + RAM",
        0x1B => "MBC5 + RAM + BATTERY",
        0x1C => "MBC5 + RUMBLE",
        0x1D => "MBC5 + RUMBLE + RAM",
        0x1E => "MBC5 + RUMBLE + RAM + BATTERY",

        _ => panic!("gbrs doesn't know the name of this cartridge's memory controller ({:#04x}).", cart_info.cart_type)
    }
}
