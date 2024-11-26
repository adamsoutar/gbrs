use crate::cartridge::{CGBSupportType, Cartridge};
use crate::config::Config;
use crate::constants::*;
use crate::gpu::Gpu;
use crate::interrupts::*;
use crate::log;
use crate::memory::memory::Memory;
use crate::registers::Registers;
use crate::{bitmatch, combine_u8, compute_equal, compute_mask, set_bit};

const BREAKPOINTS: [u16; 0] = [];
const CPU_DEBUG: bool = false;

const COND_NZ: u8 = 0b00;
const COND_Z: u8 = 0b01;
const COND_NC: u8 = 0b10;
const COND_C: u8 = 0b11;

pub enum EmulationTarget {
    // Original GameBoy
    Dmg,
    // GameBoy Color in DMG back-compat mode
    CgbDmgMode,
    // GameBoy Color in full colour mode
    CgbCgbMode,
    // GmaeBoy Advance in CGB back-compat mode
    GbaCgbMode,
}

impl EmulationTarget {
    pub fn has_cgb_features(&self) -> bool {
        match self {
            EmulationTarget::Dmg => false,
            EmulationTarget::CgbDmgMode => false,
            EmulationTarget::CgbCgbMode => true,
            EmulationTarget::GbaCgbMode => true,
        }
    }
}

// When a game supports DMG, CGB back-compat, and full colour, what should we
// run it as?
const TARGET_FOR_CGB_OPTIONAL_GAMES: EmulationTarget =
    EmulationTarget::CgbCgbMode;

fn emulation_target_for_cart_info(cart_info: &Cartridge) -> EmulationTarget {
    match cart_info.cgb_support {
        CGBSupportType::None => EmulationTarget::Dmg,
        CGBSupportType::Optional => TARGET_FOR_CGB_OPTIONAL_GAMES,
        CGBSupportType::Required => EmulationTarget::CgbCgbMode,
    }
}

pub struct Cpu {
    pub cart_info: Cartridge,
    pub mem: Memory,

    pub regs: Registers,

    pub gpu: Gpu,
    pub frame_rate: usize,

    pub ints: Interrupts,
    // When EI is executed, they're turned on after the instruction after the EI
    ime_on_pending: bool,

    // Counts the milliseconds since the CPU started
    // Based on CPU clocks. If the emulation is running too slow or too fast,
    // this will not be accurate to real-world time.
    pub ms_since_boot: usize,
    clock_counter: usize,

    halted: bool,
}

impl Cpu {
    #[inline(always)]
    fn read_next(&mut self) -> u8 {
        let byte = self.mem_read(self.regs.pc);
        // log!("Read address {:#x}, value: {:#x}", self.regs.pc, byte);
        self.regs.pc += 1;
        byte
    }
    #[inline(always)]
    fn read_next_16(&mut self) -> u16 {
        let b1 = self.read_next();
        let b2 = self.read_next();
        combine_u8!(b2, b1)
    }

    #[inline(always)]
    fn mem_write(&mut self, address: u16, value: u8) {
        self.mem
            .write(&mut self.ints, &mut self.gpu, address, value)
    }
    #[inline(always)]
    fn mem_read(&mut self, address: u16) -> u8 {
        self.mem.read(&self.ints, &self.gpu, address)
    }
    #[inline(always)]
    fn mem_write_16(&mut self, address: u16, value: u16) {
        self.mem
            .write_16(&mut self.ints, &mut self.gpu, address, value)
    }
    #[inline(always)]
    fn mem_read_16(&mut self, address: u16) -> u16 {
        self.mem.read_16(&self.ints, &self.gpu, address)
    }
    #[inline(always)]
    fn set_singular_register(&mut self, register: u8, value: u8) {
        self.regs.set_singular_register(
            register,
            value,
            &mut self.mem,
            &mut self.ints,
            &mut self.gpu,
        )
    }
    #[inline(always)]
    fn get_singular_register(&mut self, register: u8) -> u8 {
        self.regs
            .get_singular_register(register, &self.mem, &self.ints, &self.gpu)
    }

    #[inline(always)]
    fn stack_push(&mut self, value: u16) {
        self.regs.sp -= 2;
        self.mem_write_16(self.regs.sp, value);
    }
    #[inline(always)]
    fn stack_pop(&mut self) -> u16 {
        let val = self.mem_read_16(self.regs.sp);
        self.regs.sp += 2;
        val
    }

    fn condition_met(&self, condition: u8) -> bool {
        match condition {
            COND_NZ => self.regs.get_zero_flag() == 0,
            COND_Z => self.regs.get_zero_flag() == 1,
            COND_NC => self.regs.get_carry_flag() == 0,
            COND_C => self.regs.get_carry_flag() == 1,
            _ => panic!("Invalid jump condition {:#b}", condition),
        }
    }

    fn process_interrupts(&mut self) {
        // The master interrupt enable flag
        if !self.ints.ime && !self.halted {
            return;
        }

        let mut pending_ints = self.ints.flag_read();
        let enabled_ints = self.ints.enable_read();

        let any_enabled = (pending_ints & enabled_ints) > 0;
        if !any_enabled {
            return;
        }
        self.halted = false;

        if !self.ints.ime {
            return;
        }

        // log!("Enabled: {:08b}", enabled_ints);
        for i in 0..8 {
            let mask: u8 = 1 << i;
            if mask & pending_ints != 0 {
                // This interrupt is pending
                // Is it enabled?
                if mask & enabled_ints != 0 {
                    // Yes, disable all interrupts
                    self.ints.ime = false;
                    // Disable that interrupt
                    set_bit!(pending_ints, i, 0);
                    self.ints.flag_write(pending_ints);
                    // Call interrupt vector
                    self.stack_push(self.regs.pc);
                    self.regs.pc = INTERRUPT_VECTORS[i as usize];
                    return;
                }
            }
        }
    }

    // Runs enough steps to be ready to render one frame
    // (GUI implementations should get the frame from gpu.finished_frame)
    pub fn step_one_frame(&mut self) -> usize {
        let mut cycles_per_frame = CLOCK_SPEED / self.frame_rate;
        if self.mem.speed_switch.current_speed_is_double {
            cycles_per_frame *= 2;
        }

        let mut cycles = 0;
        while cycles < cycles_per_frame {
            cycles += self.step()
        }

        cycles
    }

    // Runs the CPU until the APU has filled its buffer defined by the
    // SOUND_BUFFER_SIZE constant
    pub fn step_until_full_audio_buffer(&mut self) -> usize {
        let mut cycles = 0;

        loop {
            cycles += self.step();
            if self.mem.apu.buffer_full {
                self.mem.apu.buffer_full = false;
                break;
            }
        }

        cycles
    }

    pub fn step(&mut self) -> usize {
        let mut cycles = self.single_speed_step();
        if self.mem.speed_switch.current_speed_is_double {
            cycles += self.single_speed_step();
        }
        let half_speed_cycles =
            match self.mem.speed_switch.current_speed_is_double {
                true => cycles / 2,
                false => cycles,
            };

        for _ in 0..half_speed_cycles {
            self.gpu.step(&mut self.ints, &mut self.mem);
            // Sound processing can take up to 40% of runtime
            // Some ports don't even support sound output, so we'll allow them to
            // turn off this waste of time
            #[cfg(feature = "sound")]
            self.mem.apu.step();
        }

        cycles
    }

    // Function complexity warning here is due to the massive switch statement.
    // Such a thing is expected in an emulator.
    // skipcq: RS-R1000
    pub fn single_speed_step(&mut self) -> usize {
        let p = self.ime_on_pending;

        let cycles: usize;

        if self.halted {
            cycles = 4;
        } else {
            let op = self.read_next();

            if CPU_DEBUG {
                log!(
                    "PC: {:#06x} | OPCODE: {:#04x} | {}",
                    self.regs.pc - 1,
                    op,
                    self.regs.debug_dump()
                );
            }

            for b in BREAKPOINTS.iter() {
                if self.regs.pc - 1 == *b {
                    panic!("BREAK");
                }
            }

            let v_r = (op & 0b00_11_0000) >> 4;
            let v_d = (op & 0b00_111_000) >> 3;
            let v_d_alt = op & 0b00000_111;

            // Loading from (HL) adds 4 cycles to ALU instructions
            let v_d_is_hl = v_d == 0b110;
            let v_d_alt_is_hl = v_d_alt == 0b110;

            cycles = match op {
                0 => 4,

                // LD (N),SP
                0b00001000 => {
                    // Store the stack pointer in ram
                    let addr = self.read_next_16();
                    self.mem_write_16(addr, self.regs.sp);
                    20
                },

                // LD R, N
                op if bitmatch!(op, (0, 0, _, _, 0, 0, 0, 1)) => {
                    let n = self.read_next_16();
                    self.regs.set_combined_register(v_r, n);
                    12
                },

                // ADD HL, R
                op if bitmatch!(op, (0, 0, _, _, 1, 0, 0, 1)) => {
                    let reg = self.regs.get_combined_register(v_r);
                    self.alu_add_hl(reg);
                    8
                },

                // LD (R), A
                op if bitmatch!(op, (0, 0, 0, _, 0, 0, 1, 0)) => {
                    let reg_val = self.regs.get_combined_register(v_r);
                    self.mem_write(reg_val, self.regs.a);
                    8
                },

                // LD A, (R)
                op if bitmatch!(op, (0, 0, 0, _, 1, 0, 1, 0)) => {
                    let reg_val = self.regs.get_combined_register(v_r);
                    let memval = self.mem_read(reg_val);
                    self.regs.a = memval;
                    8
                },

                // INC R
                op if bitmatch!(op, (0, 0, _, _, 0, 0, 1, 1)) => {
                    let reg_val = self.regs.get_combined_register(v_r);
                    self.regs
                        .set_combined_register(v_r, reg_val.wrapping_add(1));
                    8
                },

                // DEC R
                op if bitmatch!(op, (0, 0, _, _, 1, 0, 1, 1)) => {
                    let reg_val = self.regs.get_combined_register(v_r);
                    self.regs
                        .set_combined_register(v_r, reg_val.wrapping_sub(1));
                    8
                },

                // INC D
                op if bitmatch!(op, (0, 0, _, _, _, 1, 0, 0)) => {
                    let mut val = self.get_singular_register(v_d);
                    val = self.alu_inc(val);
                    self.set_singular_register(v_d, val);
                    4
                },

                // DEC D
                op if bitmatch!(op, (0, 0, _, _, _, 1, 0, 1)) => {
                    let mut val = self.get_singular_register(v_d);
                    val = self.alu_dec(val);
                    self.set_singular_register(v_d, val);
                    4
                },

                // LD D,N
                op if bitmatch!(op, (0, 0, _, _, _, 1, 1, 0)) => {
                    let val = self.read_next();
                    self.set_singular_register(v_d, val);
                    if v_d_is_hl {
                        12
                    } else {
                        8
                    }
                },

                // RdCA and RdA
                op if bitmatch!(op, (0, 0, 0, _, _, 1, 1, 1)) => {
                    let dir = ((op & 0b0000_1_000) >> 3) == 1;
                    let carry = ((op & 0b000_1_0000) >> 4) != 1;
                    self.alu_rotate(dir, carry);
                    4
                },

                // STOP
                0b00010000 => {
                    if self.mem.speed_switch.armed {
                        self.mem.speed_switch.execute_speed_switch();
                        // CPU halts for a really long time during speed switch
                        SPEED_SWITCH_HALT_CYCLES
                    } else {
                        log!("[WARN] STOP with un-armed CGB Speed Switch. Not used in commercial games.");
                        4
                    }
                },

                // JR N
                0b00011000 => {
                    // The displacement is signed
                    let disp = self.read_next() as i8;
                    if disp > 0 {
                        self.regs.pc = self.regs.pc.wrapping_add(disp as u16);
                    } else {
                        self.regs.pc =
                            self.regs.pc.wrapping_sub(disp.abs() as u16);
                    }
                    12
                },

                // JR F, N
                op if bitmatch!(op, (0, 0, 1, _, _, 0, 0, 0)) => {
                    let disp = self.read_next() as i8;
                    let condition = (op & 0b000_11_000) >> 3;

                    if self.condition_met(condition) {
                        // We do want to jump
                        if disp > 0 {
                            self.regs.pc =
                                self.regs.pc.wrapping_add(disp as u16);
                        } else {
                            self.regs.pc =
                                self.regs.pc.wrapping_sub(disp.abs() as u16);
                        }
                        12
                    } else {
                        8
                    }
                },

                // LD (HL+/-), A
                op if bitmatch!(op, (0, 0, 1, _, 0, 0, 1, 0)) => {
                    let is_inc = ((op & 0b000_1_0000) >> 4) == 0;
                    let mut hl = self.regs.get_hl();

                    // Write a to mem
                    self.mem_write(hl, self.regs.a);

                    // Increment/decrement
                    if is_inc {
                        hl = hl.wrapping_add(1)
                    } else {
                        hl = hl.wrapping_sub(1)
                    }
                    self.regs.set_hl(hl);

                    8
                },

                // LD A, (HL+/-)
                op if bitmatch!(op, (0, 0, 1, _, 1, 0, 1, 0)) => {
                    let is_inc = ((op & 0b000_1_0000) >> 4) == 0;
                    let mut hl = self.regs.get_hl();

                    self.regs.a = self.mem_read(hl);

                    if is_inc {
                        hl = hl.wrapping_add(1)
                    } else {
                        hl = hl.wrapping_sub(1)
                    }
                    self.regs.set_hl(hl);

                    8
                },

                // DAA
                0b00100111 => {
                    self.alu_daa();
                    4
                },

                // SCF
                0b00110111 => {
                    self.regs.set_carry_flag(1);
                    self.regs.set_half_carry_flag(0);
                    self.regs.set_operation_flag(0);
                    4
                },

                // CCF
                0b00111111 => {
                    if self.regs.get_carry_flag() == 0 {
                        self.regs.set_carry_flag(1);
                    } else {
                        self.regs.set_carry_flag(0);
                    }
                    self.regs.set_half_carry_flag(0);
                    self.regs.set_operation_flag(0);
                    4
                },

                // CPL
                0b00101111 => {
                    let value = self.regs.a;
                    self.regs.a = !value;

                    self.regs.set_half_carry_flag(1);
                    self.regs.set_operation_flag(1);

                    4
                },

                // LD D, D
                op if bitmatch!(op, (0, 1, _, _, _, _, _, _)) => {
                    let reg_val = self.get_singular_register(v_d_alt);
                    self.set_singular_register(v_d, reg_val);

                    if op == HALT_INSTRUCTION_OPCODE {
                        self.halted = true;
                        return 4;
                    }

                    if v_d_alt_is_hl {
                        8
                    } else {
                        4
                    }
                },

                // ALU A, D
                op if bitmatch!(op, (1, 0, _, _, _, _, _, _)) => {
                    let val = self.get_singular_register(v_d_alt);
                    let operation = (op & 0b00111000) >> 3;
                    self.alu(operation, val);
                    if v_d_alt_is_hl {
                        8
                    } else {
                        4
                    }
                },

                // ALU A, N
                op if bitmatch!(op, (1, 1, _, _, _, 1, 1, 0)) => {
                    let val = self.read_next();
                    let operation = (op & 0b00111000) >> 3;
                    self.alu(operation, val);
                    8
                },

                // POP R
                op if bitmatch!(op, (1, 1, _, _, 0, 0, 0, 1)) => {
                    let val = self.stack_pop();
                    self.regs.set_combined_register_alt(v_r, val);
                    12
                },

                // PUSH R
                op if bitmatch!(op, (1, 1, _, _, 0, 1, 0, 1)) => {
                    let val = self.regs.get_combined_register_alt(v_r);
                    self.stack_push(val);
                    16
                },

                // RST N
                op if bitmatch!(op, (1, 1, _, _, _, 1, 1, 1)) => {
                    let n = op & 0b00111000;
                    self.stack_push(self.regs.pc);
                    // TODO: Check if this should be 0x100 + n
                    self.regs.pc = n as u16;
                    16
                },

                // RET F
                op if bitmatch!(op, (1, 1, 0, _, _, 0, 0, 0)) => {
                    let condition = (op & 0b000_11_000) >> 3;

                    if self.condition_met(condition) {
                        self.regs.pc = self.stack_pop();
                        20
                    } else {
                        8
                    }
                },

                // RET
                0b11001001 => {
                    self.regs.pc = self.stack_pop();
                    16
                },

                // RETI
                0b11011001 => {
                    self.regs.pc = self.stack_pop();
                    // TODO: Check if this is an immediate enable
                    self.ime_on_pending = true;
                    16
                },

                // JP F, N
                op if bitmatch!(op, (1, 1, 0, _, _, 0, 1, 0)) => {
                    let condition = (op & 0b000_11_000) >> 3;
                    let address = self.read_next_16();

                    if self.condition_met(condition) {
                        self.regs.pc = address;
                        16
                    } else {
                        12
                    }
                },

                // JP N
                0b11000011 => {
                    let address = self.read_next_16();
                    self.regs.pc = address;
                    16
                },

                // CALL F, N
                op if bitmatch!(op, (1, 1, 0, _, _, 1, 0, 0)) => {
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
                },

                // CALL N
                0b11001101 => {
                    let address = self.read_next_16();
                    self.stack_push(self.regs.pc);
                    self.regs.pc = address;
                    24
                },

                // ADD SP, N
                0b11101000 => {
                    let sp = self.regs.sp;
                    let imm_raw = self.read_next();
                    let imm = i16::from(imm_raw as i8) as u16;

                    self.regs.set_carry_flag(
                        ((sp & 0xFF) + (imm & 0xFF) > 0xFF) as u8,
                    );
                    self.regs.set_half_carry_flag(
                        ((sp & 0xF) + (imm & 0xF) > 0xF) as u8,
                    );
                    self.regs.set_operation_flag(0);
                    self.regs.set_zero_flag(0);

                    self.regs.sp = sp.wrapping_add(imm);

                    16
                },

                // LD HL, SP+N
                0b11111000 => {
                    let sp = self.regs.sp;
                    let imm_raw = self.read_next();
                    let imm = i16::from(imm_raw as i8) as u16;

                    self.regs.set_carry_flag(
                        ((sp & 0xFF) + (imm & 0xFF) > 0xFF) as u8,
                    );
                    self.regs.set_half_carry_flag(
                        ((sp & 0xF) + (imm & 0xF) > 0xF) as u8,
                    );
                    self.regs.set_operation_flag(0);
                    self.regs.set_zero_flag(0);

                    self.regs.set_hl(sp.wrapping_add(imm));

                    12
                },

                // LD (FF00+N), A
                0b11100000 => {
                    let imm = self.read_next();
                    let a = self.regs.a;
                    self.mem_write(0xFF00 + imm as u16, a);

                    12
                },

                // LD A, (FF00+N)
                0b11110000 => {
                    let imm = self.read_next();
                    let val = self.mem_read(0xFF00 + imm as u16);
                    self.regs.a = val;

                    12
                },

                // LD (FF00+C), A
                0b11100010 => {
                    self.mem_write(0xFF00 + self.regs.c as u16, self.regs.a);
                    8
                },

                // LD A, (FF00+C)
                0b11110010 => {
                    self.regs.a = self.mem_read(0xFF00 + self.regs.c as u16);
                    8
                },

                // LD (N), A
                0b11101010 => {
                    let imm = self.read_next_16();
                    self.mem_write(imm, self.regs.a);
                    16
                },

                // LD A, (N)
                0b11111010 => {
                    let imm = self.read_next_16();
                    self.regs.a = self.mem_read(imm);
                    16
                },

                // JP HL
                0b11101001 => {
                    // TODO: Check if this should contain a mem read
                    self.regs.pc = self.regs.get_hl();
                    4
                },

                // LD SP, HL
                0b11111001 => {
                    self.regs.sp = self.regs.get_hl();
                    8
                },

                // DI
                0b11110011 => {
                    self.ints.ime = false;
                    self.ime_on_pending = false;
                    4
                },

                // EI
                0b11111011 => {
                    self.ime_on_pending = true;
                    4
                },

                // Prefix CB
                // More instructions are encoded by using 0xCB
                // to access an extended instruction set
                0b11001011 => {
                    let op2 = self.read_next();
                    self.execute_cb(op2)
                },

                _ => panic!(
                    "Unsupported op {:08b} ({:#04x}), PC: {} ({:#x})",
                    op,
                    op,
                    self.regs.pc - 1,
                    self.regs.pc - 1
                ),
            };
        }

        if p && self.ime_on_pending {
            self.ints.ime = true;
            self.ime_on_pending = false;
        }

        self.mem.step(cycles, &mut self.ints, self.ms_since_boot);

        self.process_interrupts();

        self.clock_counter += cycles;
        if self.clock_counter >= CLOCK_SPEED / 1000 {
            self.ms_since_boot += 1;
            self.clock_counter = 0;
        }

        return cycles;
    }

    fn execute_cb(&mut self, op: u8) -> usize {
        let v_n = (op & 0b111000) >> 3;
        let v_d = op & 0b111;
        let v_d_is_hl = v_d == 0b110;
        // Register operations take longer if D is HL
        let v_d_hl_cycles = if v_d_is_hl { 16 } else { 8 };

        match op {
            // RdC D, Rd D
            op if bitmatch!(op, (0, 0, 0, _, _, _, _, _)) => {
                let carry = op & 0b00010000 == 0;
                let right = ((op & 0b00001000) >> 3) == 1;
                let reg_val = self.get_singular_register(v_d);
                let result = self.alu_rotate_val(right, carry, reg_val);
                self.set_singular_register(v_d, result);
                v_d_hl_cycles
            },

            // SdA D
            op if bitmatch!(op, (0, 0, 1, 0, _, _, _, _)) => {
                let right = ((op & 0b00001000) >> 3) == 1;
                let reg_val = self.get_singular_register(v_d);
                let result = self.alu_special_rotate(right, reg_val);
                self.set_singular_register(v_d, result);
                v_d_hl_cycles
            },

            // SWAP D
            op if bitmatch!(op, (0, 0, 1, 1, 0, _, _, _)) => {
                let val = self.get_singular_register(v_d);
                let lower = val & 0x0F;
                let upper = (val & 0xF0) >> 4;
                let new = (lower << 4) | upper;
                self.set_singular_register(v_d, new);

                self.regs.set_zero_flag((new == 0) as u8);
                self.regs.set_half_carry_flag(0);
                self.regs.set_carry_flag(0);
                self.regs.set_operation_flag(0);

                v_d_hl_cycles
            },

            // SRL D
            op if bitmatch!(op, (0, 0, 1, 1, 1, _, _, _)) => {
                let reg_val = self.get_singular_register(v_d);
                let result = self.alu_srl(reg_val);
                self.set_singular_register(v_d, result);
                v_d_hl_cycles
            },

            // BIT N, D
            op if bitmatch!(op, (0, 1, _, _, _, _, _, _)) => {
                let val = self.get_singular_register(v_d);
                let mask = 1 << v_n;

                self.regs.set_zero_flag(((val & mask) == 0) as u8);
                self.regs.set_operation_flag(0);
                self.regs.set_half_carry_flag(1);

                v_d_hl_cycles
            },

            // RES N, D
            op if bitmatch!(op, (1, 0, _, _, _, _, _, _)) => {
                let mut val = self.get_singular_register(v_d);
                set_bit!(val, v_n, 0);
                self.set_singular_register(v_d, val);
                v_d_hl_cycles
            },

            // SET N, D
            op if bitmatch!(op, (1, 1, _, _, _, _, _, _)) => {
                let mut val = self.get_singular_register(v_d);
                set_bit!(val, v_n, 1);
                self.set_singular_register(v_d, val);
                v_d_hl_cycles
            },

            _ => panic!("Unsupported CB_op {:08b} ({:#04x})", op, op),
        }
    }

    pub fn from_config(config: Config) -> Cpu {
        let cart_info =
            Cartridge::parse(&config.rom.bytes, config.rom.path.clone());
        let emulation_target = emulation_target_for_cart_info(&cart_info);

        Cpu {
            mem: Memory::from_info(
                cart_info.clone(),
                config.rom,
                &emulation_target,
            ),
            cart_info,
            regs: Registers::new(&emulation_target),

            gpu: Gpu::new(emulation_target.has_cgb_features()),
            frame_rate: DEFAULT_FRAME_RATE,

            ints: Interrupts::new(),
            ime_on_pending: false,

            ms_since_boot: 0,
            clock_counter: 0,

            halted: false,
        }
    }
}
