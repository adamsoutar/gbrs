// Gameboy Link Cable
// Mostly a stub, although some parts have to be emulated *somewhat* accurately
// to emulate fussy games like Alleyway
use crate::gameboy::constants::*;
use crate::gameboy::interrupts::{Interrupts, InterruptReason};

pub struct SerialCable {
  transfer_data_byte: u8,
  transfer_control_byte: u8
}

impl SerialCable {
  pub fn read (&self, address: u16) -> u8 {
    match address {
      LINK_CABLE_SB => self.transfer_data_byte,
      LINK_CABLE_SC => self.transfer_control_byte,
      _ => unreachable!()
    }
  }

  pub fn write (&mut self, address: u16, value: u8) {
    match address {
      LINK_CABLE_SB => {
        // println!("{:#04x} was written to serial DATA", value);
        self.transfer_data_byte = value
      },
      LINK_CABLE_SC => {
        // println!("{:#04x} was written to serial CONTROL", value);
        self.transfer_control_byte = value;
      },
      _ => unreachable!()
    }
  }

  pub fn step (&mut self, ints: &mut Interrupts) {
    self.transfer_data_byte = self.transfer_data_byte << 1;

    if self.transfer_control_byte & 0b1000_0000 > 0 {
      // Game requested a serial transfer. We will acknowledge.
      self.transfer_control_byte &= 0b0111_1111;
      ints.raise_interrupt(InterruptReason::Serial);
    }
  }

  pub fn new () -> SerialCable {
    SerialCable {
      transfer_data_byte: 0,
      transfer_control_byte: 0
    }
  }
}