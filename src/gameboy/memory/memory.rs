use crate::gameboy::constants::*;
use crate::gameboy::memory::ram::Ram;
use crate::gameboy::memory::rom::Rom;
use crate::gameboy::gpu::Gpu;
use crate::gameboy::interrupts::*;
use crate::gameboy::helpers::*;
use crate::gameboy::joypad::Joypad;

pub struct Memory {
    // Public 'cause the GUI reads it for the game title
    pub rom: Rom,
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

    pub joypad: Joypad
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
            INTERRUPT_ENABLE_ADDRESS => ints.enable_read(),
            INTERRUPT_FLAG_ADDRESS => ints.flag_read(),
            ROM_START ..= ROM_END => self.rom.read(address - ROM_START),
            VRAM_START ..= VRAM_END => self.vram.read(address - VRAM_START),
            WRAM_START ..= WRAM_END => self.wram.read(address - WRAM_START),
            OAM_START ..= OAM_END => gpu.raw_read(address),

            UNUSABLE_MEMORY_START ..= UNUSABLE_MEMORY_END => 0,

            // STUB: No real link cable support
            LINK_CABLE_SB | LINK_CABLE_SC => 0,
            // STUB: No sound support yet
            APU_START ..= APU_END => 0,

            LCD_DATA_START ..= LCD_DATA_END => gpu.raw_read(address),
            HRAM_START ..= HRAM_END => self.hram.read(address - HRAM_START),

            0xFF00 => self.joypad.read(),

            // Timers
            0xFF04 => self.timer_divider,
            0xFF05 => self.timer_counter,
            0xFF06 => self.timer_modulo,
            0xFF07 => self.timer_control,

            _ => panic!("Unsupported memory read at {:#06x}", address)
        }
    }

    pub fn write (&mut self, ints: &mut Interrupts, gpu: &mut Gpu, address: u16, value: u8) {
        match address {
            INTERRUPT_ENABLE_ADDRESS => ints.enable_write(value),
            INTERRUPT_FLAG_ADDRESS => ints.flag_write(value),

            // Games without a MBC (the only ones we support at the moment) ignore writes
            ROM_START ..= ROM_END => {},

            // TODO: Disable writing to VRAM if GPU is reading it
            VRAM_START ..= VRAM_END => self.vram.write(address - VRAM_START, value),
            WRAM_START ..= WRAM_END => self.wram.write(address - WRAM_START, value),
            OAM_START ..= OAM_END => gpu.raw_write(address, value),

            // TETRIS writes here.. for some reason?
            UNUSABLE_MEMORY_START ..= UNUSABLE_MEMORY_END => {},

            // STUB: No link cable support
            LINK_CABLE_SB => println!("{:#04x} was written to the link cable", value),
            LINK_CABLE_SC => println!("{:#04x} was written to the link cable control field", value),
            // STUB: No sound support yet
            APU_START ..= APU_END => println!("{:#04x} was written to the APU at {:#06x}", value, address),

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

            _ => panic!("Unsupported memory write at {:#06x} (value: {:#04x})", address, value)
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
            hram: Ram::new(HRAM_SIZE),
            timer_divider_increase: 0,
            timer_divider: 0,
            timer_counter_increase: 0,
            timer_counter: 0,
            timer_control: 0,
            timer_modulo: 0,
            joypad: Joypad::new()
        }
    }
}
