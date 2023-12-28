use crate::constants::*;
use crate::cpu::EmulationTarget;
use crate::memory::ram::Ram;
use crate::memory::rom::Rom;
use crate::gpu::Gpu;
use crate::interrupts::*;
use crate::{split_u16, combine_u8};
use crate::joypad::Joypad;
use crate::cartridge::Cartridge;
use crate::memory::mbcs::*;
use crate::sound::apu::APU;
use crate::serial_cable::SerialCable;
use crate::log;
use crate::colour::palette_ram::PaletteRam;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

// TODO: Rename this to something more appropriate
//       (I've seen an emu call a similar struct 'Interconnect')
pub struct Memory {
    cgb_features: bool,

    mbc: Box<dyn MBC>,

    // TODO: Move VRAM to GPU?
    vram: Ram,
    wram_banks: [Ram; 8],
    // On DMG, this is always 1. On CGB, it's 1-7 inclusive
    upper_wram_bank: usize,
    hram: Ram,
    // Used in CGB mode only
    palette_ram: PaletteRam,

    serial_cable: SerialCable,

    timer_divider_increase: u16,
    timer_divider: u8,

    timer_counter_increase: u32,
    timer_counter: u8,

    timer_modulo: u8,

    timer_control: u8,

    pub joypad: Joypad,

    pub apu: APU
}

fn create_wram_banks () -> [Ram; 8] {
    [
        Ram::new(WRAM_BANK_SIZE),
        Ram::new(WRAM_BANK_SIZE),
        Ram::new(WRAM_BANK_SIZE),
        Ram::new(WRAM_BANK_SIZE),

        Ram::new(WRAM_BANK_SIZE),
        Ram::new(WRAM_BANK_SIZE),
        Ram::new(WRAM_BANK_SIZE),
        Ram::new(WRAM_BANK_SIZE),
    ]
}

impl Memory {
    // Memory has a step command for timers & MBCs
    pub fn step (&mut self, cycles: usize, ints: &mut Interrupts, ms_since_boot: usize) {
        // These two timers are safe to implement like this vs per-cycle
        // because the CPU will never do more than about 16 cycles per step,
        // let alone >256
        self.timer_divider_increase += cycles as u16;
        if self.timer_divider_increase >= 256 {
            self.timer_divider_increase -= 256;
            self.timer_divider = self.timer_divider.wrapping_add(1);
        }

        let enabled = (self.timer_control >> 2) == 1;
        if enabled {
            self.timer_counter_increase += cycles as u32;

            let step = match self.timer_control & 0b11 {
                0b00 => 1024,
                0b01 => 16,
                0b10 => 64,
                0b11 => 256,
                _ => unreachable!()
            };

            while self.timer_counter_increase >= step {
                self.timer_counter = self.timer_counter.wrapping_add(1);
                if self.timer_counter == 0 {
                    self.timer_counter = self.timer_modulo;
                    ints.raise_interrupt(InterruptReason::Timer);
                }
                self.timer_counter_increase -= step;
            }
        }

        self.serial_cable.step(ints, cycles);

        self.mbc.step(ms_since_boot);

        // Sound processing can take up to 40% of runtime
        // Some ports don't even support sound output, so we'll allow them to
        // turn off this waste of time
        #[cfg(feature = "sound")]
        for _ in 0..cycles {
            self.apu.step();
        }
    }

    #[inline(always)]
    pub fn read (&self, ints: &Interrupts, gpu: &Gpu, address: u16) -> u8 {
        match address {
            // Cartridge memory starts at the 0 address
            0 ..= MBC_ROM_END => self.mbc.read(address),

            VRAM_START ..= VRAM_END => self.vram.read(address - VRAM_START),

            MBC_RAM_START ..= MBC_RAM_END => self.mbc.ram_read(address - MBC_RAM_START),

            WRAM_LOWER_BANK_START ..= WRAM_LOWER_BANK_END =>
                self.wram_banks[0].read(address - WRAM_LOWER_BANK_START),
            WRAM_UPPER_BANK_START ..= WRAM_UPPER_BANK_END =>
                self.wram_banks[self.upper_wram_bank].read(address - WRAM_UPPER_BANK_START),
            // TODO: How does upper echo RAM work with CGB bank switching?
            ECHO_RAM_START ..= ECHO_RAM_END => self.read(ints, gpu, address - (ECHO_RAM_START - WRAM_LOWER_BANK_START)),

            OAM_START ..= OAM_END => gpu.raw_read(address),

            UNUSABLE_MEMORY_START ..= UNUSABLE_MEMORY_END => 0xFF,

            LINK_CABLE_SB | LINK_CABLE_SC => self.serial_cable.read(address),

            APU_START ..= APU_END => self.apu.read(address),

            LCD_DATA_START ..= LCD_DATA_END => gpu.raw_read(address),
            CGB_PALETTE_DATA_START ..= CGB_PALETTE_DATA_END => self.palette_ram.raw_read(address),
            HRAM_START ..= HRAM_END => self.hram.read(address - HRAM_START),

            0xFF00 => self.joypad.read(),

            // Timers
            0xFF04 => self.timer_divider,
            0xFF05 => self.timer_counter,
            0xFF06 => self.timer_modulo,
            0xFF07 => self.timer_control,

            0xFF70 => self.upper_wram_bank as u8,

            INTERRUPT_ENABLE_ADDRESS => ints.enable_read(),
            INTERRUPT_FLAG_ADDRESS => ints.flag_read(),

            _ => { log!("Unsupported memory read at {:#06x}", address); 0xFF }
        }
    }

    #[inline(always)]
    pub fn write (&mut self, ints: &mut Interrupts, gpu: &mut Gpu, address: u16, value: u8) {
        match address {
            0 ..= MBC_ROM_END => self.mbc.write(address, value),

            // TODO: Disable writing to VRAM if GPU is reading it
            VRAM_START ..= VRAM_END => self.vram.write(address - VRAM_START, value),

            MBC_RAM_START ..= MBC_RAM_END => self.mbc.ram_write(address - MBC_RAM_START, value),

            WRAM_LOWER_BANK_START ..= WRAM_LOWER_BANK_END =>
                self.wram_banks[0].write(address - WRAM_LOWER_BANK_START, value),
            WRAM_UPPER_BANK_START ..= WRAM_UPPER_BANK_END =>
                self.wram_banks[self.upper_wram_bank].write(address - WRAM_UPPER_BANK_START, value),
            ECHO_RAM_START ..= ECHO_RAM_END => self.write(ints, gpu, address - (ECHO_RAM_START - WRAM_LOWER_BANK_START), value),

            OAM_START ..= OAM_END => gpu.raw_write(address, value, ints),

            // TETRIS writes here.. due to a bug
            UNUSABLE_MEMORY_START ..= UNUSABLE_MEMORY_END => {},

            LINK_CABLE_SB | LINK_CABLE_SC => self.serial_cable.write(address, value),

            APU_START ..= APU_END => self.apu.write(address, value),

            LCD_DATA_START ..= LCD_DATA_END => gpu.raw_write(address, value, ints),
            CGB_PALETTE_DATA_START ..= CGB_PALETTE_DATA_END => self.palette_ram.raw_write(address, value),
            HRAM_START ..= HRAM_END => self.hram.write(address - HRAM_START, value),

            0xFF00 => self.joypad.write(value),

            // Timers
            0xFF04 => self.timer_divider = 0,
            // NOTE: This goes to 0 when written to, not to value
            0xFF05 => self.timer_counter = 0,
            0xFF06 => self.timer_modulo = value,
            0xFF07 => self.timer_control = value,

            // Upper WRAM bank select
            0xFF70 => {
                if !self.cgb_features  { return; }
                let mut desired_bank = value & 0x07;
                if desired_bank == 0 { desired_bank = 1; }
                self.upper_wram_bank = desired_bank as usize;
            },

            // TETRIS also writes here, Sameboy doesn't seem to care
            0xFF7F => {},

            INTERRUPT_ENABLE_ADDRESS => ints.enable_write(value),
            INTERRUPT_FLAG_ADDRESS => ints.flag_write(value),

            _ => log!("Unsupported memory write at {:#06x} (value: {:#04x})", address, value)
        }
    }

    #[inline(always)]
    pub fn read_16(&self, ints: &Interrupts, gpu: &Gpu, address: u16) -> u16 {
        combine_u8!(self.read(ints, gpu, address + 1), self.read(ints, gpu, address))
    }
    #[inline(always)]
    pub fn write_16(&mut self, ints: &mut Interrupts, gpu: &mut Gpu, address: u16, value: u16) {
        let (b1, b2) = split_u16!(value);
        self.write(ints, gpu, address, b1);
        self.write(ints, gpu, address + 1, b2);
    }

    pub fn from_info (cart_info: Cartridge, rom: Rom, target: &EmulationTarget) -> Memory {
        Memory {
            cgb_features: target.has_cgb_features(),
            mbc: mbc_from_info(cart_info, rom),
            vram: Ram::new(VRAM_SIZE),
            wram_banks: create_wram_banks(),
            upper_wram_bank: 1,
            hram: Ram::new(HRAM_SIZE),
            palette_ram: PaletteRam::new(&target),
            serial_cable: SerialCable::new(),
            timer_divider_increase: 0,
            timer_divider: 0,
            timer_counter_increase: 0,
            timer_counter: 0,
            timer_control: 0b00000010,
            timer_modulo: 0,
            joypad: Joypad::new(),
            apu: APU::new()
        }
    }
}
