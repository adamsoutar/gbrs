use crate::gameboy::helpers::*;

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,

    pub sp: u16,
    pub pc: u16
}
impl Registers {
    fn set_flag(&mut self, flag_index: u8, bit: u8) {
        set_bit(&mut self.f, flag_index, bit)
    }
    pub fn set_carry_flag (&mut self, bit: u8) {
        self.set_flag(0, bit)
    }
    pub fn set_half_carry_flag (&mut self, bit: u8) {
        self.set_flag(1, bit)
    }
    pub fn set_operation_flag (&mut self, bit: u8) {
        self.set_flag(2, bit)
    }
    pub fn set_zero_flag (&mut self, bit: u8) {
        self.set_flag(3, bit)
    }

    pub fn set_flags(&mut self, zero: u8, operation: u8, half_carry: u8, carry: u8) {
        self.set_carry_flag(carry);
        self.set_half_carry_flag(half_carry);
        self.set_operation_flag(operation);
        self.set_zero_flag(zero);
    }

    fn get_flag(&self, flag_index: u8) -> u8 {
        (self.f >> (4 + flag_index)) & 0x1
    }
    pub fn get_carry_flag (&self) -> u8 {
        self.get_flag(0)
    }
    pub fn get_half_carry_flag (&self) -> u8 {
        self.get_flag(1)
    }
    pub fn get_operation_flag (&self) -> u8 {
        self.get_flag(2)
    }
    pub fn get_zero_flag (&self) -> u8 {
        self.get_flag(3)
    }

    pub fn get_af (&self) -> u16 {
        // TODO: Check these are the right way around
        combine_u8(self.a, self.f)
    }
    pub fn set_af (&mut self, value: u16) {
        let (b1, b2) = split_u16(value);
        self.a = b1; self.f = b2;
    }

    pub fn get_bc (&self) -> u16 {
        combine_u8(self.b, self.c)
    }
    pub fn set_bc (&mut self, value: u16) {
        let (b1, b2) = split_u16(value);
        self.b = b1; self.c = b2;
    }

    pub fn get_de (&self) -> u16 {
        combine_u8(self.d, self.e)
    }
    pub fn set_de (&mut self, value: u16) {
        let (b1, b2) = split_u16(value);
        self.d = b1; self.e = b2;
    }

    pub fn get_hl (&self) -> u16 {
        combine_u8(self.h, self.l)
    }
    pub fn set_hl (&mut self, value: u16) {
        let (b1, b2) = split_u16(value);
        self.h = b1; self.l = b2;
    }

    // These are left to right from the "GoldenCrystal Gameboy Z80 CPU Opcodes" PDF
    // TODO: Rename this
    pub fn set_combined_register (&mut self, register: u8, value: u16) {
        match register {
            0b00 => self.set_bc(value),
            0b01 => self.set_de(value),
            0b10 => self.set_hl(value),
            0b11 => self.sp = value,
            _ => panic!("Invalid combined register set")
        }
    }
    pub fn get_combined_register (&mut self, register: u8) -> u16 {
        match register {
            0b00 => self.get_bc(),
            0b01 => self.get_de(),
            0b10 => self.get_hl(),
            0b11 => self.sp,
            _ => panic!("Invalid combined register get")
        }
    }

    pub fn set_singular_register (&mut self, register: u8, value: u8) {
        match register {
            0b000 => self.b = value,
            0b001 => self.c = value,
            0b010 => self.d = value,
            0b011 => self.e = value,
            0b100 => self.h = value,
            0b101 => self.l = value,
            0b110 => panic!("D to (HL) unsupported"), // TODO
            0b111 => self.a = value,
            _ => panic!("Invalid singular register set")
        }
    }

    pub fn get_singular_register (&mut self, register: u8) -> u8 {
        match register {
            0b000 => self.b,
            0b001 => self.c,
            0b010 => self.d,
            0b011 => self.e,
            0b100 => self.h,
            0b101 => self.l,
            0b110 => panic!("D to (HL) unsupported"), // TODO
            0b111 => self.a,
            _ => panic!("Invalid singular register get")
        }
    }

    pub fn new () -> Registers {
        Registers {
            a: 0, b: 0, c: 0, d: 0, e: 0, f: 0, h: 0, l: 0,
            sp: 0xFFFE, pc: 0x100
        }
    }
}
