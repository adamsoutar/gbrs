use crate::gameboy::memory::memory::Memory;
use crate::gameboy::helpers::*;
use crate::{bitmatch, compute_mask, compute_equal};
use crate::gameboy::registers::Registers;

const ALU_ADD: u8 = 0b000;
const ALU_ADC: u8 = 0b001;
const ALU_SUB: u8 = 0b010;
const ALU_SBC: u8 = 0b011;
const ALU_AND: u8 = 0b100;
const ALU_XOR: u8 = 0b101;
const ALU_OR: u8 = 0b110;
const ALU_CP: u8 = 0b111;

const COND_NZ: u8 = 0b00;
const COND_Z: u8 = 0b01;
const COND_NC: u8 = 0b10;
const COND_C: u8 = 0b11;

pub struct Cpu {
    mem: Memory,
    regs: Registers
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

    // TODO: Cleanup
    fn alu(&mut self, operation: u8, n: u8) {
        let a = self.regs.a;
        let c = self.regs.get_carry_flag();

        match operation {
            ALU_ADD => {
                // ADD
                let res = a.wrapping_add(n);
                self.regs.set_carry_flag((a as u16 + n as u16 > 0xFF) as u8);
                self.regs.set_half_carry_flag(((a & 0x0F) + (n & 0x0F) > 0x0F) as u8);
                self.regs.set_zero_flag((res == 0) as u8);
                self.regs.set_operation_flag(0);
                self.regs.a = res;
            },
            ALU_ADC => {
                // ADC
                let res = a.wrapping_add(n).wrapping_add(c);
                self.regs.set_carry_flag((a as u16 + n as u16 > 0xFF) as u8);
                self.regs.set_half_carry_flag(((a & 0x0F) + (n & 0x0F) + c > 0x0F) as u8);
                self.regs.set_zero_flag((res == 0) as u8);
                self.regs.set_operation_flag(0);
                self.regs.a = res;
            },
            ALU_SUB => {
                // SUB
                let res = a.wrapping_sub(n);
                self.regs.set_carry_flag((a < n) as u8);
                self.regs.set_half_carry_flag(((a & 0x0F) < (n & 0x0F)) as u8);
                self.regs.set_operation_flag(1);
                self.regs.set_zero_flag((res == 0) as u8);
                self.regs.a = res;
            },
            ALU_SBC => {
                // SBC
                let res = a.wrapping_sub(n).wrapping_sub(c);
                self.regs.set_carry_flag(((a as u16) < (n as u16 + c as u16)) as u8);
                self.regs.set_half_carry_flag(((a & 0x0F) < (n & 0x0F) + c) as u8);
                self.regs.set_operation_flag(1);
                self.regs.set_zero_flag((res == 0) as u8);
                self.regs.a = res;
            },
            ALU_AND => {
                // AND
                let res = a & n;
                self.regs.set_carry_flag(0);
                self.regs.set_half_carry_flag(1);
                self.regs.set_operation_flag(0);
                self.regs.set_zero_flag((res == 0) as u8);
                self.regs.a = res;
            },
            ALU_XOR => {
                // XOR
                let res = a ^ n;
                self.regs.set_carry_flag(0);
                self.regs.set_half_carry_flag(0);
                self.regs.set_operation_flag(0);
                self.regs.set_zero_flag((res == 0) as u8);
                self.regs.a = res;
            },
            ALU_OR => {
                // OR
                let res = a | n;
                self.regs.set_carry_flag(0);
                self.regs.set_half_carry_flag(0);
                self.regs.set_operation_flag(0);
                self.regs.set_zero_flag((res == 0) as u8);
                self.regs.a = res;
            },
            ALU_CP => {
                // CP ("Compare")
                // It's a subtraction in terms of flags, but it throws away the result
                self.alu(ALU_SUB, n);
                self.regs.a = a;
            },
            _ => panic!("Unsupported ALU operation {:b}", operation)
        }
    }

    fn alu_dec (&mut self, n: u8) -> u8 {
        let r = n.wrapping_sub(1);
        self.regs.set_half_carry_flag((n.trailing_zeros() >= 4) as u8);
        self.regs.set_operation_flag(1);
        self.regs.set_zero_flag((r == 0) as u8);
        println!("Just did a dec");
        self.regs.debug_dump();
        println!("See dump ^");
        r
    }
    fn alu_inc(&mut self, n: u8) -> u8 {
        let r = n.wrapping_add(1);
        self.regs.set_half_carry_flag(((n & 0x0f) + 0x01 > 0x0f) as u8);
        self.regs.set_operation_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }
    fn alu_add_hl(&mut self, n: u16) {
        let hl = self.regs.get_hl();
        let r = hl.wrapping_add(n);

        self.regs.set_carry_flag((hl > 0xffff - n) as u8);
        self.regs.set_half_carry_flag(((hl & 0x0fff) + (n & 0x0fff) > 0x0fff) as u8);
        self.regs.set_operation_flag(0);

        self.regs.set_hl(r);
    }

    fn condition_met (&self, condition: u8) -> bool {
        match condition {
            // NZ
            COND_NZ => self.regs.get_zero_flag() == 0,
            // Z
            COND_Z => self.regs.get_zero_flag() == 1,
            // NC
            COND_NC => self.regs.get_carry_flag() == 0,
            // C
            COND_C => self.regs.get_carry_flag() == 1,
            _ => panic!("Invalid jump condition {:#b}", condition)
        }
    }

    // TODO: Flags
    fn step (&mut self) -> usize {
        println!("PC: {:#x}", self.regs.pc);
        self.regs.debug_dump();

        let op = self.read_next();
        println!("OPCODE: {:#x}", op);

        let v_r = (op & 0b00_11_0000) >> 4;
        let v_d = (op & 0b00_111_000) >> 3;
        let v_d_alt = op & 0b00000_111;

        // Loading from (HL) adds 4 cycles to ALU instructions
        let v_d_is_hl = (v_d & 0b110) == 0b110;

        match op {
            0 => { 4 },
            // TODO: LD (N),SP

            // LD R, N
            op if bitmatch!(op, (0,0,_,_,0,0,0,1)) => {
                let n = self.read_next_16();
                self.regs.set_combined_register(v_r, n);
                12
            }

            // ADD HL, R
            op if bitmatch!(op, (0,0,_,_,1,0,0,1)) => {
                let reg = self.regs.get_combined_register(v_r);
                self.alu_add_hl(reg);
                8
            }

            // LD (R), A
            op if bitmatch!(op, (0,0,0,_,0,0,1,0)) => {
                let reg_val = self.regs.get_combined_register(v_r);
                let memval = self.mem.read(reg_val);
                self.regs.a = memval;
                8
            }

            // LD A, (R)
            op if bitmatch!(op, (0,0,0,_,1,0,1,0)) => {
                let reg_val = self.regs.get_combined_register(v_r);
                self.mem.write(reg_val, self.regs.a);
                8
            }

            // INC R
            op if bitmatch!(op, (0,0,_,_,0,0,1,1)) => {
                let reg_val = self.regs.get_combined_register(v_r);
                self.regs.set_combined_register(v_r, reg_val.wrapping_add(1));
                8
            }

            // DEC R
            op if bitmatch!(op, (0,0,_,_,1,0,1,1)) => {
                let reg_val = self.regs.get_combined_register(v_r);
                self.regs.set_combined_register(v_r, reg_val.wrapping_sub(1));
                8
            }

            // INC D
            op if bitmatch!(op, (0,0,_,_,_,1,0,0)) => {
                let mut val = self.regs.get_singular_register(v_d);
                val = self.alu_inc(val);
                self.regs.set_singular_register(v_d, val);
                4
            }

            // DEC D
            op if bitmatch!(op, (0,0,_,_,_,1,0,1)) => {
                let mut val = self.regs.get_singular_register(v_d);
                println!("ALU DEC");
                val = self.alu_dec(val);
                self.regs.set_singular_register(v_d, val);
                4
            }

            // LD D,N
            op if bitmatch!(op, (0,0,_,_,_,1,1,0)) => {
                let val = self.read_next();
                self.regs.set_singular_register(v_d, val);
                8
            },

            // JR N
            0b00011000 => {
                // The displacement is signed
                let disp = self.read_next() as i8;
                if disp > 0 {
                    self.regs.pc = self.regs.pc.wrapping_add(disp as u16);
                } else {
                    self.regs.pc = self.regs.pc.wrapping_sub(disp.abs() as u16);
                }
                12
            },

            // JR F, N
            op if bitmatch!(op, (0,0,1,_,_,0,0,0)) => {
                let disp = self.read_next() as i8;
                let condition = (op & 0b000_11_000) >> 3;

                if self.condition_met(condition) {
                    // We do want to jump
                    if disp > 0 {
                        self.regs.pc = self.regs.pc.wrapping_add(disp as u16);
                    } else {
                        self.regs.pc = self.regs.pc.wrapping_sub(disp.abs() as u16);
                    }
                    12
                } else {
                    8
                }
            },

            // LD (HL+/-), A
            op if bitmatch!(op, (0,0,1,_,0,0,1,0)) => {
                let is_inc = ((op & 0b000_1_0000) >> 4) == 0;
                let mut hl = self.regs.get_hl();

                // Load from mem into a
                let val = self.mem.read(hl);
                self.regs.a = val;

                // Increment/decrement
                if is_inc { hl = hl.wrapping_add(1) } else { hl = hl.wrapping_sub(1) }
                self.regs.set_hl(hl);

                8
            }

            // ALU A, D
            op if bitmatch!(op, (1,0,_,_,_,_,_,_)) => {
                let val = self.regs.get_singular_register(v_d_alt);
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
                println!("JUMP TO {:x}", address);
                self.regs.pc = address;
                16
            }

            _ => panic!("Unsupported op {:b} ({:#x}), PC: {} ({:#x})", op, op, self.regs.pc - 1, self.regs.pc - 1)
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
            regs: Registers::new()
        }
    }
}
