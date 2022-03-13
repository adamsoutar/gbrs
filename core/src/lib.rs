#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod memory;
pub mod constants;
pub mod cpu;
pub mod helpers;
pub mod registers;
pub mod interrupts;
pub mod gpu;
pub mod lcd;
pub mod cartridge;
pub mod joypad;
pub mod apu;
pub mod serial_cable;
