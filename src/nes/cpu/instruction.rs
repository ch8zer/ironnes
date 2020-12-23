use super::addressing::AddressingMode;
use crate::nes::bus::Bus;
use crate::nes::memory::*;

// Instruction Table
include!(concat!(env!("OUT_DIR"), "/instruction_lookup.rs"));

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: u8,
    mnemonic: String,
    pub bytes: u8,
    pub cycles: usize,
    pub can_cross_page: bool,
    pub addr_mode: AddressingMode,
}

impl Instruction {
    // TODO turn into const&
    pub fn lookup(opcode: u8) -> Self {
        lookup_instr(opcode)
    }

    pub fn new(
        opcode: u8,
        mnemonic: &str,
        bytes: u8,
        cycles: usize,
        can_cross_page: bool,
        addr_mode: AddressingMode,
    ) -> Self {
        Self {
            opcode,
            mnemonic: mnemonic.to_string(),
            bytes,
            cycles,
            can_cross_page,
            addr_mode,
        }
    }

    fn illegal(opcode: u8) -> Self {
        Self {
            opcode,
            mnemonic: "ILLEGAL".to_string(),
            bytes: 1,
            cycles: 0,
            can_cross_page: false,
            addr_mode: AddressingMode::Illegal,
        }
    }

    pub fn print(&self, pc: Addr, bus: &mut Bus) -> String {
        let p1 = bus.cpu_load((pc as usize) + 1).unwrap();
        let p2 = bus.cpu_load((pc as usize) + 2).unwrap();
        match self.addr_mode {
            AddressingMode::Accumulator => format!("{:02x}       {} A", self.opcode, self.mnemonic),
            AddressingMode::Immediate => format!(
                "{:02x} {:02x}    {} #${:02x}",
                self.opcode, p1, self.mnemonic, p1
            ),
            AddressingMode::Absolute => format!(
                "{:02x} {:02x} {:02x} {} ${:02x}{:02x}",
                self.opcode, p1, p2, self.mnemonic, p2, p1
            ),
            AddressingMode::AbsoluteX => format!(
                "{:02x} {:02x} {:02x} {} ${:02x}{:02x}, X",
                self.opcode, p1, p2, self.mnemonic, p2, p1
            ),
            AddressingMode::AbsoluteY => format!(
                "{:02x} {:02x} {:02x} {} ${:02x}{:02x}, Y",
                self.opcode, p1, p2, self.mnemonic, p2, p1
            ),
            AddressingMode::Indirect => format!(
                "{:02x} {:02x} {:02x} {} ${:02x}{:02x}",
                self.opcode, p1, p2, self.mnemonic, p2, p1
            ),
            AddressingMode::IndirectX => format!(
                "{:02x} {:02x}    {} (${:02x},X)",
                self.opcode, p1, self.mnemonic, p1
            ),
            AddressingMode::IndirectY => format!(
                "{:02x} {:02x}    {} (${:02x},Y)",
                self.opcode, p1, self.mnemonic, p1
            ),
            AddressingMode::ZeroPage => format!(
                "{:02x} {:02x}    {} ${:02x}",
                self.opcode, p1, self.mnemonic, p1
            ),
            AddressingMode::ZeroPageX => format!(
                "{:02x} {:02x}    {} ${:02x},X",
                self.opcode, p1, self.mnemonic, p1
            ),
            AddressingMode::ZeroPageY => format!(
                "{:02x} {:02x}    {} ${:02x},Y",
                self.opcode, p1, self.mnemonic, p1
            ),
            AddressingMode::Relative => format!(
                "{:02x} {:02x}    {} ${:02x}",
                self.opcode, p1, self.mnemonic, p1
            ),
            AddressingMode::Illegal => format!(
                "{:02x}      {} ${:02x}",
                self.opcode, self.mnemonic, self.opcode
            ),
            _ => format!("{:02x}       {} ", self.opcode, self.mnemonic),
        }
    }
}
