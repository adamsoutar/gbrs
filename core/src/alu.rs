// CPU Arithmetic Logic Unit
use crate::cpu::Cpu;

const ALU_ADD: u8 = 0b000;
const ALU_ADC: u8 = 0b001;
const ALU_SUB: u8 = 0b010;
const ALU_SBC: u8 = 0b011;
const ALU_AND: u8 = 0b100;
const ALU_XOR: u8 = 0b101;
const ALU_OR: u8 = 0b110;
const ALU_CP: u8 = 0b111;

impl Cpu {
    pub fn alu(&mut self, operation: u8, n: u8) {
        let a = self.regs.a;
        let c = self.regs.get_carry_flag();

        match operation {
            ALU_ADD => {
                // ADD
                let res = a.wrapping_add(n);
                self.regs.set_carry_flag((a as u16 + n as u16 > 0xFF) as u8);
                self.regs.set_half_carry_flag(
                    ((a & 0x0F) + (n & 0x0F) > 0x0F) as u8,
                );
                self.regs.set_zero_flag((res == 0) as u8);
                self.regs.set_operation_flag(0);
                self.regs.a = res;
            },
            ALU_ADC => {
                // ADC
                let res = a.wrapping_add(n).wrapping_add(c);
                self.regs.set_carry_flag(
                    (a as u16 + n as u16 + c as u16 > 0xFF) as u8,
                );
                self.regs.set_half_carry_flag(
                    ((a & 0x0F) + (n & 0x0F) + c > 0x0F) as u8,
                );
                self.regs.set_zero_flag((res == 0) as u8);
                self.regs.set_operation_flag(0);
                self.regs.a = res;
            },
            ALU_SUB => {
                // SUB
                let res = a.wrapping_sub(n);
                self.regs.set_carry_flag((a < n) as u8);
                self.regs
                    .set_half_carry_flag(((a & 0x0F) < (n & 0x0F)) as u8);
                self.regs.set_operation_flag(1);
                self.regs.set_zero_flag((res == 0) as u8);
                self.regs.a = res;
            },
            ALU_SBC => {
                // SBC
                let res = a.wrapping_sub(n).wrapping_sub(c);
                self.regs
                    .set_carry_flag(((a as u16) < (n as u16 + c as u16)) as u8);
                self.regs
                    .set_half_carry_flag(((a & 0x0F) < (n & 0x0F) + c) as u8);
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
            _ => panic!("Unsupported ALU operation {:b}", operation),
        }
    }

    pub fn alu_dec(&mut self, n: u8) -> u8 {
        let r = n.wrapping_sub(1);
        self.regs
            .set_half_carry_flag((n.trailing_zeros() >= 4) as u8);
        self.regs.set_operation_flag(1);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }
    pub fn alu_inc(&mut self, n: u8) -> u8 {
        let r = n.wrapping_add(1);
        self.regs
            .set_half_carry_flag(((n & 0x0f) + 0x01 > 0x0f) as u8);
        self.regs.set_operation_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }
    pub fn alu_add_hl(&mut self, n: u16) {
        let hl = self.regs.get_hl();
        let r = hl.wrapping_add(n);

        self.regs.set_carry_flag((hl > 0xffff - n) as u8);
        self.regs
            .set_half_carry_flag(((hl & 0x0fff) + (n & 0x0fff) > 0x0fff) as u8);
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
    fn alu_rl(&mut self, n: u8) -> u8 {
        let c = (n & 0b10000000) >> 7;
        let r = (n << 1) | self.regs.get_carry_flag();
        self.regs.set_carry_flag(c);
        self.regs.set_operation_flag(0);
        self.regs.set_half_carry_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }
    fn alu_rrc(&mut self, n: u8) -> u8 {
        let c = n & 1;
        let r = (n >> 1) | (c << 7);
        self.regs.set_carry_flag(c);
        self.regs.set_operation_flag(0);
        self.regs.set_half_carry_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }
    fn alu_rr(&mut self, n: u8) -> u8 {
        let c = n & 1;
        let r = (n >> 1) | (self.regs.get_carry_flag() << 7);
        self.regs.set_carry_flag(c);
        self.regs.set_half_carry_flag(0);
        self.regs.set_operation_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }

    fn alu_sla(&mut self, n: u8) -> u8 {
        let c = (n & 0x80) >> 7;
        let r = n << 1;
        self.regs.set_carry_flag(c);
        self.regs.set_half_carry_flag(0);
        self.regs.set_operation_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }
    fn alu_sra(&mut self, n: u8) -> u8 {
        let c = n & 1;
        let r = (n >> 1) | (n & 0x80);
        self.regs.set_carry_flag(c);
        self.regs.set_half_carry_flag(0);
        self.regs.set_operation_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }
    pub fn alu_special_rotate(&mut self, right: bool, n: u8) -> u8 {
        if right {
            self.alu_sra(n)
        } else {
            self.alu_sla(n)
        }
    }

    pub fn alu_srl(&mut self, n: u8) -> u8 {
        let c = n & 1;
        let r = n >> 1;
        self.regs.set_carry_flag(c);
        self.regs.set_half_carry_flag(0);
        self.regs.set_operation_flag(0);
        self.regs.set_zero_flag((r == 0) as u8);
        r
    }

    pub fn alu_rotate_val(&mut self, right: bool, carry: bool, a: u8) -> u8 {
        if !right {
            if carry {
                self.alu_rlc(a)
            } else {
                self.alu_rl(a)
            }
        } else {
            if carry {
                self.alu_rrc(a)
            } else {
                self.alu_rr(a)
            }
        }
    }
    pub fn alu_rotate(&mut self, right: bool, carry: bool) {
        let a = self.regs.a;
        self.regs.a = self.alu_rotate_val(right, carry, a);
    }

    // DAA is proper weird. For this one, I had to look at:
    // https://github.com/mohanson/gameboy/blob/master/src/cpu.rs#L325
    pub fn alu_daa(&mut self) {
        let mut a = self.regs.a;

        let mut adjust = if self.regs.get_carry_flag() == 1 {
            0x60
        } else {
            0
        };

        if self.regs.get_half_carry_flag() == 1 {
            adjust |= 0x06;
        }

        if self.regs.get_operation_flag() == 0 {
            if a & 0x0f > 0x09 {
                adjust |= 0x06;
            };
            if a > 0x99 {
                adjust |= 0x60;
            };
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }

        self.regs.set_carry_flag((adjust >= 0x60) as u8);
        self.regs.set_half_carry_flag(0);
        self.regs.set_zero_flag((a == 0) as u8);

        self.regs.a = a;
    }
}
