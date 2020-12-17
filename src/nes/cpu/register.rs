use crate::bitset::BitSet;
use crate::nes::memory;
use log::*;
use std::fmt;

#[allow(dead_code)]
pub enum Flags {
    /// Carry
    C = 0,
    /// Zero
    Z = 1,
    /// Interrupts Enabled
    I = 2,
    /// Decimal
    D = 3,
    /// BRK (executing an interrupt)
    B = 4,
    /// oVerflow
    V = 6,
    /// Negative
    N = 7,
}

pub struct Registers {
    // Should be a 16 bit registers
    pub pc: memory::Addr,
    pub sp: memory::Addr,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub flags: BitSet,
}

impl Registers {
    const P_DEFAULT: u8 = 0b00100100;

    pub fn new() -> Self {
        trace!("Initializing Registers");
        Self {
            pc: 0xc000,
            sp: 0xfd,
            a: 0,
            x: 0,
            y: 0,
            flags: BitSet::new(Self::P_DEFAULT),
        }
    }

    pub fn get_status(&self) -> u8 {
        self.flags.cast()
    }

    pub fn set_status(&mut self, s: u8) {
        self.flags = BitSet::new(s)
    }

    pub fn set_n(&mut self, x: u16) {
        self.set_flag(Flags::N, (x & 0x80) != 0);
    }

    pub fn set_z(&mut self, x: u16) {
        self.set_flag(Flags::Z, x == 0);
    }

    pub fn get_flag(&mut self, f: Flags) -> bool {
        self.flags.get(f as u8)
    }

    pub fn set_flag(&mut self, f: Flags, s: bool) {
        match s {
            true => self.flags.set(f as u8),
            _ => self.flags.clear(f as u8),
        }
    }

    pub fn clear_status(&mut self) {
        self.flags = BitSet::new(Self::P_DEFAULT);
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PC {:04x} SP {:02x} A {:02x} X {:02x} Y {:02x} P {:02x}",
            self.pc,
            self.sp,
            self.a,
            self.x,
            self.y,
            self.get_status()
        )
    }
}
