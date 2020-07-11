use crate::gameboy::memory::memory::Memory;
use crate::gameboy::helpers::*;
use crate::{bitmatch, compute_mask, compute_equal};
use crate::gameboy::registers::Registers;

const BREAK_AT_INSTRUCTION: usize = 10;

pub struct Cpu {
    mem: Memory,
    regs: Registers,

    ins_count: usize
}

impl Cpu {
    fn read_next (&mut self) -> u8 {
        let byte = self.mem.read(self.regs.pc);
        // println!("Read address {:#x}, value: {:#x}", self.regs.pc, byte);
        self.regs.pc += 1;
        byte
    }
    fn read_next_16 (&mut self) -> u16 {
        let b1 = self.read_next();
        let b2 = self.read_next();
        combine_u8(b2, b1)
    }

    fn alu(&mut self, operation: u8, value: u8) {

    }

    // TODO: Flags
    fn step (&mut self) -> usize {
        let op = self.read_next();

        let v_r = (op & 0b00_11_0000) >> 4;
        let v_d = (op & 0b00_111_000) >> 3;
        // Loading from (HL) adds 4 cycles to ALU instructions
        let v_d_is_hl = (v_d & 0b110) == 0b110;

        if self.ins_count == BREAK_AT_INSTRUCTION {
            panic!("BREAK AT INSTRUCTION");
        }
        self.ins_count += 1;

        match op {
            0 => { 4 },
            // TODO: LD (N),SP

            // LD R, N
            op if bitmatch!(op, (0,0,_,_,0,0,0,1)) => {
            // 0b00_00_0001 | 0b00_01_0001 | 0b00_10_0001 | 0b00_11_0001 => {
                let n = self.read_next_16();
                self.regs.set_combined_register(v_r, n);
                12
            }

            // ADD HL, R
            op if bitmatch!(op, (0,0,_,_,1,0,0,1)) => {
            // 0b00_00_1001 | 0b00_01_1001 | 0b00_10_1001 | 0b00_11_1001 => {
                let hl = self.regs.get_hl();
                let reg = self.regs.get_combined_register(v_r);
                self.regs.set_hl(hl + reg);
                8
            }

            // LD (R), A
            op if bitmatch!(op, (0,0,0,_,0,0,1,0)) => {
            // 0b000_0_0010 | 0b000_1_0010 => {
                let reg_val = self.regs.get_combined_register(v_r);
                let memval = self.mem.read(reg_val);
                self.regs.a = memval;
                8
            }

            // LD A, (R)
            op if bitmatch!(op, (0,0,0,_,1,0,1,0)) => {
            // 0b000_0_1010 | 0b000_1_1010 => {
                let reg_val = self.regs.get_combined_register(v_r);
                self.mem.write(reg_val, self.regs.a);
                8
            }

            // INC R
            op if bitmatch!(op, (0,0,_,_,0,0,1,1)) => {
            // 0b00_00_0011 | 0b00_01_0011 | 0b00_10_0011 | 0b00_11_0011 => {
                let reg_val = self.regs.get_combined_register(v_r);
                self.regs.set_combined_register(v_r, reg_val + 1);
                8
            }

            // DEC R
            op if bitmatch!(op, (0,0,_,_,1,0,1,1)) => {
            // 0b00_00_1011 | 0b00_01_1011 | 0b00_10_1011 | 0b00_11_1011 => {
                let reg_val = self.regs.get_combined_register(v_r);
                self.regs.set_combined_register(v_r, reg_val - 1);
                8
            }

            // INC D
            op if bitmatch!(op, (0,0,_,_,_,1,0,0)) => {
            // 0b00_000_100 | 0b00_001_100 | 0b00_010_100 | 0b00_011_100 | 0b00_100_100 |
            // 0b00_101_100 | 0b00_110_100 | 0b00_111_100 => {
                let val = self.regs.get_singular_register(v_d);
                self.regs.set_singular_register(v_d, val + 1);
                4
            }

            // DEC D
            op if bitmatch!(op, (0,0,_,_,_,1,0,1)) => {
            // 0b00_000_101 | 0b00_001_101 | 0b00_010_101 | 0b00_011_101 | 0b00_100_101 |
            // 0b00_101_101 | 0b00_110_101 | 0b00_111_101 => {
                let val = self.regs.get_singular_register(v_d);
                self.regs.set_singular_register(v_d, val - 1);
                4
            }

            // ...

            // HALT
            0b01110110 => panic!("CPU HALT"),

            // ALU A, D
            op if bitmatch!(op, (1,0,_,_,_,_,_,_)) => {
                let val = self.regs.get_singular_register(v_d);
                let operation = (op & 0b00111000) >> 3;
                self.alu(operation, val);
                if v_d_is_hl { 8 } else { 4 }
            }

            // ALU A, N
            op if bitmatch!(op, (1,1,_,_,_,1,1,0)) => {
                let val = self.read_next();
                let operation = (op & 0b00111000) >> 3;
                self.alu(operation, val);
                8
            }

            // JP N
            0b11000011 => {
                let address = self.read_next_16();
                println!("Jumping to {:x}", address);
                self.regs.pc = address;
                16
            }

            _ => panic!("Unsupported op {:b} ({:#x}), PC: {}", op, op, self.regs.pc)
        }
    }

    pub fn run (&mut self) {
        loop {
            self.step();
        }
    }

    pub fn from_rom (rom_path: String) -> Cpu {
        Cpu {
            mem: Memory::from_rom(rom_path),
            regs: Registers::new(),

            ins_count: 0
        }
    }
}
