#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod alu;
pub mod callbacks;
pub mod cartridge;
pub mod cgb_dma;
pub mod colour; // innit bruv
pub mod config;
pub mod constants;
pub mod cpu;
pub mod gpu;
pub mod helpers;
pub mod interrupts;
pub mod joypad;
pub mod lcd;
pub mod memory;
pub mod registers;
pub mod serial_cable;
pub mod sound;
