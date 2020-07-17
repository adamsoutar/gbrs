use crate::gameboy::memory::rom::Rom;
use crate::gameboy::cartridge::Cartridge;

pub trait MBC {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    fn ram_read(&self, address: u16) -> u8;
    fn ram_write(&mut self, address: u16, value: u8);
}

mod none;
mod mbc1;

pub fn mbc_from_info(cart_info: Cartridge, rom: Rom) -> Box<dyn MBC> {
    println!("Loading game \"{}\"", cart_info.title);
    println!("Cart type: {:#04x}", cart_info.cart_type);
    println!("ROM size: {}KB", cart_info.rom_size / 1024);
    println!("RAM size: {}KB", cart_info.ram_size / 1024);

    match cart_info.cart_type {
        0x00 => Box::new(none::MBCNone::new(rom)),
        0x01 ..= 0x03 => Box::new(mbc1::MBC1::new(cart_info, rom)),
        _ => panic!("gbrs doesn't support this cartridge's memory controller.")
    }
}
