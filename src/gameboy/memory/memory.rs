use crate::gameboy::constants::*;
use crate::gameboy::memory::ram::Ram;
use crate::gameboy::memory::rom::Rom;
use crate::gameboy::gpu::Gpu;
use crate::gameboy::interrupts::*;
use crate::gameboy::helpers::*;
use crate::gameboy::joypad::Joypad;
use crate::gameboy::cartridge::Cartridge;
use crate::gameboy::memory::mbcs::*;
use crate::gameboy::apu::APU;

// TODO: Rename this to something more appropriate
//       (I've seen an emu call a similar struct 'Interconnect')
pub struct Memory {
    mbc: Box<dyn MBC>,

    // TODO: Move VRAM to GPU
    vram: Ram,
    wram: Ram,
    hram: Ram,

    timer_divider_increase: u16,
    timer_divider: u8,

    timer_counter_increase: u32,
    timer_counter: u8,

    timer_modulo: u8,

    timer_control: u8,

    pub joypad: Joypad,

    pub apu: APU
}

impl Memory {
    fn get_counter_increase (&self) -> u32 {
        let enabled = (self.timer_control >> 2) == 1;
        if !enabled { return 0 }

        match self.timer_control & 0b11 {
            0b00 => 64,
            0b01 => 1,
            0b10 => 4,
            0b11 => 16,
            _ => panic!()
        }
        //
        // match self.timer_control & 0b11 {
        //     0b00 => 1,
        //     0b01 => 64,
        //     0b10 => 16,
        //     0b11 => 4,
        //     _ => panic!()
        // }
    }

    // Memory has a step command for timers
    pub fn step (&mut self, cycles: usize, ints: &mut Interrupts) {
        for _ in 0..cycles {
            self.timer_divider_increase += 1;
            if self.timer_divider_increase == 256 {
                self.timer_divider_increase = 0;
                self.timer_divider = self.timer_divider.wrapping_add(1);
            }

            let inc = self.get_counter_increase();
            self.timer_counter_increase += inc;
            if self.timer_counter_increase == 262144 {
                self.timer_counter_increase = 0;
                self.timer_counter = self.timer_counter.wrapping_add(1);
                // If it overflowed
                if self.timer_counter == 0 {
                    self.timer_counter = self.timer_modulo;
                    ints.raise_interrupt(InterruptReason::Timer);
                }
            }
        }
    }

    pub fn read (&self, ints: &Interrupts, gpu: &Gpu, address: u16) -> u8 {
        match address {
            // Cartridge memory starts at the 0 address
            0 ..= MBC_ROM_END => self.mbc.read(address),

            VRAM_START ..= VRAM_END => self.vram.read(address - VRAM_START),

            MBC_RAM_START ..= MBC_RAM_END => self.mbc.ram_read(address - MBC_RAM_START),

            WRAM_START ..= WRAM_END => self.wram.read(address - WRAM_START),
            OAM_START ..= OAM_END => gpu.raw_read(address),

            UNUSABLE_MEMORY_START ..= UNUSABLE_MEMORY_END => 0xFF,

            // STUB: No real link cable support
            LINK_CABLE_SB | LINK_CABLE_SC => 0,

            APU_START ..= APU_END => self.apu.read(address),

            LCD_DATA_START ..= LCD_DATA_END => gpu.raw_read(address),
            HRAM_START ..= HRAM_END => self.hram.read(address - HRAM_START),

            0xFF00 => self.joypad.read(),

            // Timers
            0xFF04 => self.timer_divider,
            0xFF05 => self.timer_counter,
            0xFF06 => self.timer_modulo,
            0xFF07 => self.timer_control,

            INTERRUPT_ENABLE_ADDRESS => ints.enable_read(),
            INTERRUPT_FLAG_ADDRESS => ints.flag_read(),

            _ => panic!("Unsupported memory read at {:#06x}", address)
        }
    }

    pub fn write (&mut self, ints: &mut Interrupts, gpu: &mut Gpu, address: u16, value: u8) {
        match address {
            0 ..= MBC_ROM_END => self.mbc.write(address, value),

            // TODO: Disable writing to VRAM if GPU is reading it
            VRAM_START ..= VRAM_END => self.vram.write(address - VRAM_START, value),

            MBC_RAM_START ..= MBC_RAM_END => self.mbc.ram_write(address - MBC_RAM_START, value),

            WRAM_START ..= WRAM_END => self.wram.write(address - WRAM_START, value),
            OAM_START ..= OAM_END => gpu.raw_write(address, value),

            // TETRIS writes here.. due to a bug
            UNUSABLE_MEMORY_START ..= UNUSABLE_MEMORY_END => {},

            // STUB: No link cable support
            LINK_CABLE_SB => {}, //println!("{:#04x} was written to the link cable", value),
            LINK_CABLE_SC => {}, //println!("{:#04x} was written to the link cable control field", value),

            APU_START ..= APU_END => self.apu.write(address, value),

            LCD_DATA_START ..= LCD_DATA_END => gpu.raw_write(address, value),
            HRAM_START ..= HRAM_END => self.hram.write(address - HRAM_START, value),

            0xFF00 => self.joypad.write(value),

            // Timers
            0xFF04 => self.timer_divider = 0,
            // TODO: Does this go to 0 when written?
            0xFF05 => self.timer_counter = value,
            0xFF06 => self.timer_modulo = value,
            0xFF07 => self.timer_control = value,

            // TETRIS also writes here, Sameboy doesn't seem to care
            0xFF7F => {},

            INTERRUPT_ENABLE_ADDRESS => ints.enable_write(value),
            INTERRUPT_FLAG_ADDRESS => ints.flag_write(value),

            _ => println!("Unsupported memory write at {:#06x} (value: {:#04x})", address, value)
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

    pub fn from_info (cart_info: Cartridge, rom: Rom) -> Memory {
        Memory {
            mbc: mbc_from_info(cart_info, rom),
            vram: Ram::new(VRAM_SIZE),
            wram: Ram::new(WRAM_SIZE),
            hram: Ram::new(HRAM_SIZE),
            timer_divider_increase: 0,
            timer_divider: 0,
            timer_counter_increase: 0,
            timer_counter: 0,
            timer_control: 0,
            timer_modulo: 0,
            joypad: Joypad::new(),
            apu: APU::new()
        }
    }
}
