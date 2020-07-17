use crate::gameboy::memory::rom::Rom;
use crate::gameboy::cartridge::Cartridge;

pub trait MBC {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

pub mod none;

pub fn mbc_from_info(cart_info: Cartridge, rom: Rom) -> Box<dyn MBC> {
    println!("Loading game \"{}\"", cart_info.title);
    println!("Cart type: {:#04x}", cart_info.cart_type);
    println!("ROM size: {:#04x}", cart_info.rom_size);
    println!("RAM size: {:#04x}", cart_info.ram_size);

    match cart_info.cart_type {
        0x00 => Box::new(none::MBCNone::new(rom)),
        _ => panic!("gbrs doesn't support this cartridge's memory controller.")
    }
}
