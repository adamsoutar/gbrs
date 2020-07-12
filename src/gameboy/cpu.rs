use crate::gameboy::memory::memory::Memory;
use crate::gameboy::helpers::*;
use crate::{bitmatch, compute_mask, compute_equal};
use crate::gameboy::registers::Registers;
use crate::gameboy::interrupts::*;
use crate::gameboy::gpu::Gpu;

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
    pub mem: Memory,
    regs: Registers,

    pub gpu: Gpu,

    ints: Interrupts,
    // When EI is executed, they're turned on after the instruction after the EI
    ime_on_pending: bool
}

impl Cpu {
    fn read_next (&mut self) -> u8 {
        let byte = self.mem_read(self.regs.pc);
        // println!("Read address {:#x}, value: {:#x}", self.regs.pc, byte);
        self.regs.pc += 1;
        byte
    }
    fn read_next_16 (&mut self) -> u16 {
        let b1 = self.read_next();
        let b2 = self.read_next();
        combine_u8(b2, b1)
    }

    fn mem_write (&mut self, address: u16, value: u8) {
        self.mem.write(&mut self.ints, address, value)
    }
    fn mem_read (&mut self, address: u16) -> u8 {
        self.mem.read(&self.ints, address)
    }
    fn mem_write_16 (&mut self, address: u16, value: u16) {
        self.mem.write_16(&mut self.ints, address, value)
    }
    fn mem_read_16 (&mut self, address: u16) -> u16 {
        self.mem.read_16(&self.ints, address)
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

    // R for "Rotate" (Bitshift)
    fn alu_rlc(&mut self, n: u8) -> u8 {
        let c = (n & 0b10000000) >> 7;
        let r = (n << 1) | c;
        self.regs.set_carry_flag(c);
        self.regs.set_operation_flag(0);
        self.regs.set_half_carry_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }
    fn alu_rl (&mut self, n: u8) -> u8 {
        let c = (n & 0b10000000) >> 7;
        let r = (n << 1) | self.regs.get_carry_flag();
        self.regs.set_carry_flag(c);
        self.regs.set_operation_flag(0);
        self.regs.set_half_carry_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }
    fn alu_rrc (&mut self, n: u8) -> u8 {
        let c = n & 1;
        let r = (n >> 1) | (c << 7);
        self.regs.set_carry_flag(c);
        self.regs.set_operation_flag(0);
        self.regs.set_half_carry_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }
    fn alu_rr (&mut self, n: u8) -> u8 {
        let c = n & 1;
        let r = (n >> 1) | (self.regs.get_carry_flag() << 7);
        self.regs.set_carry_flag(c);
        self.regs.set_half_carry_flag(0);
        self.regs.set_operation_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }

    fn alu_rotate (&mut self, left: bool, carry: bool) {
        let a = self.regs.a;

        self.regs.a = if left {
            if carry { self.alu_rlc(a) }
            else { self.alu_rl(a) }
        } else {
            if carry { self.alu_rrc(a) }
            else { self.alu_rr(a) }
        }
    }

    fn stack_push (&mut self, value: u16) {
        // TODO: Check this order
        self.regs.sp -= 2;
        self.mem_write_16(self.regs.sp, value);
    }
    fn stack_pop (&mut self) -> u16 {
        let val = self.mem_read_16(self.regs.sp);
        self.regs.sp += 2;
        val
    }

    fn condition_met (&self, condition: u8) -> bool {
        match condition {
            COND_NZ => self.regs.get_zero_flag() == 0,
            COND_Z => self.regs.get_zero_flag() == 1,
            COND_NC => self.regs.get_carry_flag() == 0,
            COND_C => self.regs.get_carry_flag() == 1,
            _ => panic!("Invalid jump condition {:#b}", condition)
        }
    }

    pub fn step (&mut self) -> usize {
        println!("PC: {:#06x}", self.regs.pc);
        self.regs.debug_dump();

        let op = self.read_next();
        println!("OPCODE: {:#04x}", op);

        let v_r = (op & 0b00_11_0000) >> 4;
        let v_d = (op & 0b00_111_000) >> 3;
        let v_d_alt = op & 0b00000_111;

        // Loading from (HL) adds 4 cycles to ALU instructions
        let v_d_is_hl = (v_d & 0b110) == 0b110;

        match op {
            0 => { 4 },

            // LD (N),SP
            0b00001000 => {
                // Store the stack pointer in ram
                let addr = self.read_next_16();
                self.mem_write_16(addr, self.regs.sp);
                20
            }

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
                let memval = self.mem_read(reg_val);
                self.regs.a = memval;
                8
            }

            // LD A, (R)
            op if bitmatch!(op, (0,0,0,_,1,0,1,0)) => {
                let reg_val = self.regs.get_combined_register(v_r);
                self.mem_write(reg_val, self.regs.a);
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

            // RdCA and RdA
            op if bitmatch!(op, (0,0,0,_,_,1,1,1)) => {
                let dir = ((op & 0b0000_1_000) >> 3) == 1;
                let carry = ((op & 0b000_1_0000) >> 4) == 1;
                self.alu_rotate(dir, carry);
                4
            }

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
                let val = self.mem_read(hl);
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

            // POP R
            op if bitmatch!(op, (1,1,_,_,0,0,0,1)) => {
                let val = self.stack_pop();
                self.regs.set_combined_register_alt(v_r, val);
                12
            }

            // PUSH R
            op if bitmatch!(op, (1,1,_,_,0,1,0,1)) => {
                let val = self.regs.get_combined_register_alt(v_r);
                self.stack_push(val);
                16
            }

            // RST N
            op if bitmatch!(op, (1,1,_,_,_,1,1,1)) => {
                let n = op & 0b00111000;
                self.stack_push(self.regs.pc);
                // TODO: Check if this should be 0x100 + n
                self.regs.pc = n as u16;
                16
            }

            // RET F
            op if bitmatch!(op, (1,1,0,_,_,0,0,0)) => {
                let condition = (op & 0b000_11_000) >> 3;

                if self.condition_met(condition) {
                    self.regs.pc = self.stack_pop();
                    20
                } else {
                    8
                }
            }

            // RET
            0b11001001 => {
                self.regs.pc = self.stack_pop();
                16
            }

            // RETI
            0b11011001 => {
                self.regs.pc = self.stack_pop();
                // TODO: Check if this is an immediate enable
                self.ime_on_pending = true;
                16
            }

            // JP F, N
            op if bitmatch!(op, (1,1,0,_,_,0,1,0)) => {
                let condition = (op & 0b000_11_000) >> 3;
                let address = self.read_next_16();

                if self.condition_met(condition) {
                    self.regs.pc = address;
                    16
                } else {
                    12
                }
            }

            // JP N
            0b11000011 => {
                let address = self.read_next_16();
                self.regs.pc = address;
                16
            }

            // CALL F, N
            op if bitmatch!(op, (1,1,0,_,_,0,1,0)) => {
                let address = self.read_next_16();
                // TODO: Pull out 0b000_11_000 into a common pattern like v_r
                let condition = (op & 0b000_11_000) >> 3;

                if self.condition_met(condition) {
                    self.stack_push(self.regs.pc);
                    self.regs.pc = address;
                    24
                } else {
                    12
                }
            }

            // CALL N
            0b11001101 => {
                let address = self.read_next_16();
                self.stack_push(self.regs.pc);
                self.regs.pc = address;
                24
            }


            // LD (FF00+N), A
            0b11100000 => {
                let imm = self.read_next();
                let a = self.regs.a;
                self.mem_write(0xFF00 + imm as u16, a);

                12
            }

            // DI
            0b11110011 => {
                self.ints.ime = false;
                self.ime_on_pending = false;
                4
            }

            // EI
            0b11111011 => {
                self.ime_on_pending = true;
                4
            }

            _ => panic!("Unsupported op {:b} ({:#x}), PC: {} ({:#x})", op, op, self.regs.pc - 1, self.regs.pc - 1)
        }
    }

    pub fn run (&mut self) {
        loop {
            let p = self.ime_on_pending;

            self.step();

            // If IME's been pending for 2 steps, we turn it on
            if p && self.ime_on_pending {
                self.ints.ime = true;
                self.ime_on_pending = false;
            }
        }
    }

    pub fn from_rom (rom_path: String) -> Cpu {
        Cpu {
            mem: Memory::from_rom(rom_path),
            regs: Registers::new(),

            gpu: Gpu::new(),

            ints: Interrupts::new(),
            ime_on_pending: false
        }
    }
}
