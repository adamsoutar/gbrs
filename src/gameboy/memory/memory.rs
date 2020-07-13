use crate::gameboy::constants::*;
use crate::gameboy::memory::ram::Ram;
use crate::gameboy::memory::rom::Rom;
use crate::gameboy::gpu::Gpu;
use crate::gameboy::interrupts::*;
use crate::gameboy::helpers::*;

pub struct Memory {
    rom: Rom,
    vram: Ram,
    wram: Ram,
    hram: Ram
}

impl Memory {
    pub fn read (&self, ints: &Interrupts, gpu: &Gpu, address: u16) -> u8 {
        match address {
            INTERRUPT_ENABLE_ADDRESS => ints.enable_read(),
            INTERRUPT_FLAG_ADDRESS => ints.flag_read(),
            ROM_START ..= ROM_END => self.rom.read(address - ROM_START),
            VRAM_START ..= VRAM_END => self.vram.read(address - VRAM_START),
            WRAM_START ..= WRAM_END => self.wram.read(address - WRAM_START),

            // STUB: No real link cable support
            LINK_CABLE_SB | LINK_CABLE_SC => 0,
            // STUB: No sound support yet
            APU_START ..= APU_END => 0,

            LCD_DATA_START ..= LCD_DATA_END => gpu.raw_read(address),
            HRAM_START ..= HRAM_END => self.hram.read(address - HRAM_START),
            _ => panic!("Unsupported memory read at {} ({:#x})", address, address)
        }
    }

    pub fn write (&mut self, ints: &mut Interrupts, gpu: &mut Gpu, address: u16, value: u8) {
        match address {
            INTERRUPT_ENABLE_ADDRESS => ints.enable_write(value),
            INTERRUPT_FLAG_ADDRESS => ints.flag_write(value),
            ROM_START ..= ROM_END => panic!("ROM is read only"),
            // TODO: Disable writing to VRAM if GPU is reading it
            VRAM_START ..= VRAM_END => self.vram.write(address - VRAM_START, value),
            WRAM_START ..= WRAM_END => self.wram.write(address - WRAM_START, value),

            // STUB: No link cable support
            LINK_CABLE_SB => println!("{:#04x} was written to the link cable", value),
            LINK_CABLE_SC => println!("{:#04x} was written to the link cable control field", value),
            // STUB: No sound support yet
            APU_START ..= APU_END => println!("{:#04x} was written to the APU at {:#06x}", value, address),

            LCD_DATA_START ..= LCD_DATA_END => gpu.raw_write(address, value),
            HRAM_START ..= HRAM_END => self.hram.write(address - HRAM_START, value),
            _ => panic!("Unsupported memory write at {} ({:#x})", address, address)
        }
    }

    pub fn read_16(&self, ints: &Interrupts, gpu: &Gpu, address: u16) -> u16 {
        combine_u8(self.read(ints, gpu, address + 1), self.read(ints, gpu, address))
    }
    pub fn write_16(&mut self, ints: &mut Interrupts, gpu: &mut Gpu, address: u16, value: u16) {
        let (b1, b2) = split_u16(value);
        self.write(ints, gpu, address, b1);
        self.write(ints, gpu, address + 1, b2);
    }

    pub fn from_rom (rom_path: String) -> Memory {
        Memory {
            rom: Rom::from_file(rom_path),
            vram: Ram::new(VRAM_SIZE),
            wram: Ram::new(WRAM_SIZE),
            hram: Ram::new(HRAM_SIZE)
        }
    }
}
