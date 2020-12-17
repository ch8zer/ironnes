use super::register::Registers;
use crate::error::*;
use crate::nes::memory::{Addr, Memory};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Illegal,
    Unknown,
}

impl FromStr for AddressingMode {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let res = match input {
            "IMP" => AddressingMode::Implied,
            "ACC" => AddressingMode::Accumulator,
            "IMM" => AddressingMode::Immediate,
            "ABS" => AddressingMode::Absolute,
            "ABSX" => AddressingMode::AbsoluteX,
            "ABSY" => AddressingMode::AbsoluteY,
            "IND" => AddressingMode::Indirect,
            "INDX" => AddressingMode::IndirectX,
            "INDY" => AddressingMode::IndirectY,
            "ZP" => AddressingMode::ZeroPage,
            "ZPX" => AddressingMode::ZeroPageX,
            "ZPY" => AddressingMode::ZeroPageY,
            "REL" => AddressingMode::Relative,
            "ILL" => AddressingMode::Illegal,
            _ => AddressingMode::Unknown,
        };

        Ok(res)
    }
}

impl AddressingMode {
    pub fn load_operand(&self, reg: &Registers, mem: &Memory) -> IronNesResult<u16> {
        // TODO better performance, don't return Addr.
        // maybe an enum or template the return?
        match self {
            AddressingMode::Accumulator => Ok(reg.a as u16),
            AddressingMode::Immediate => Ok(mem.load(reg.pc - 1)?.into()),
            AddressingMode::Absolute => mem.load16(reg.pc - 2),
            AddressingMode::AbsoluteX => Ok((reg.x as u16).wrapping_add(mem.load16(reg.pc - 2)?)),
            AddressingMode::AbsoluteY => Ok((reg.y as u16).wrapping_add(mem.load16(reg.pc - 2)?)),
            AddressingMode::ZeroPage => Ok(mem.load(reg.pc - 1)?.into()),
            AddressingMode::ZeroPageX => {
                Ok(((mem.load(reg.pc - 1)?.wrapping_add(reg.x)) & 0xff).into())
            }
            AddressingMode::ZeroPageY => {
                Ok(((mem.load(reg.pc - 1)?.wrapping_add(reg.y)) & 0xff).into())
            }
            AddressingMode::Indirect => {
                let imm: Addr = mem.load16(reg.pc - 2)?;
                mem.load16(imm)
            }
            AddressingMode::IndirectX => {
                let addr = mem.load(reg.pc - 1)? as Addr;
                let addr_idx = addr.wrapping_add(reg.x as Addr);
                let imm = addr_idx & 0xff;
                Ok(mem.load16(imm)?)
            }
            AddressingMode::IndirectY => {
                let imm: Addr = mem.load(reg.pc - 1)?.into();
                Ok(mem.load16(imm)?.wrapping_add(reg.y as Addr))
            }
            AddressingMode::Relative => {
                let x: Addr = mem.load(reg.pc - 1)?.into();

                // Convert unsigned 8 bit to signed
                let x = ((x as i16) ^ 0x80) - 0x80;
                let x = x as u16;

                Ok(reg.pc.wrapping_add(x))
            }
            _ => Err(IronNesError::IllegalInstruction),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_absolute() -> IronNesResult<()> {
        let mut r = Registers::new();
        r.pc = 0xd005;
        r.x = 5;
        r.y = 0xff;
        let mut m = Memory::new();

        m.store16(r.pc - 2, 0xc000)?;

        let instr = AddressingMode::Absolute;
        let val = instr.load_operand(&r, &m)?;
        assert_eq!(0xc000, val);

        let instr = AddressingMode::AbsoluteX;
        let val = instr.load_operand(&r, &m)?;
        assert_eq!(0xc005, val);

        let instr = AddressingMode::AbsoluteY;
        let val = instr.load_operand(&r, &m)?;
        assert_eq!(0xc0ff, val);

        Ok(())
    }

    #[test]
    fn test_mode_zeropage() -> IronNesResult<()> {
        let mut r = Registers::new();
        r.pc = 0xd005;
        r.x = 5;
        r.y = 0xf;
        let mut m = Memory::new();

        m.store(r.pc - 1, 0xc0)?;

        let instr = AddressingMode::ZeroPage;
        let val = instr.load_operand(&r, &m)?;
        assert_eq!(0xc0, val);

        let instr = AddressingMode::ZeroPageX;
        let val = instr.load_operand(&r, &m)?;
        assert_eq!(0xc5, val);

        let instr = AddressingMode::ZeroPageY;
        let val = instr.load_operand(&r, &m)?;
        assert_eq!(0xcf, val);

        Ok(())
    }

    #[test]
    fn test_mode_relative() -> IronNesResult<()> {
        let mut r = Registers::new();
        r.pc = 0xc005;
        let mut m = Memory::new();

        let instr = AddressingMode::Relative;

        m.store(r.pc - 1, 0x3)?;
        let val = instr.load_operand(&r, &m)?;
        assert_eq!(0xc008, val);

        Ok(())
    }

    #[test]
    fn test_mode_relative_wrap() -> IronNesResult<()> {
        let mut r = Registers::new();
        r.pc = 0xc72a + 2;
        let mut m = Memory::new();

        let instr = AddressingMode::Relative;
        m.store(r.pc - 1, 0xe0)?;
        let val = instr.load_operand(&r, &m)?;
        assert_eq!(0xc70c, val);

        Ok(())
    }

    #[test]
    fn test_mode_indirect() -> IronNesResult<()> {
        let mut r = Registers::new();
        r.pc = 0xc400;
        let mut m = Memory::new();
        let instr = AddressingMode::Indirect;

        // Immediate value of the op
        m.store16(r.pc - 2, 0xd15f)?;

        // Actual value in memory
        m.store16(0xd15f, 0x3076)?;

        let val = instr.load_operand(&r, &m)?;
        assert_eq!(0x3076, val);
        Ok(())
    }

    #[test]
    fn test_mode_indirectx() -> IronNesResult<()> {
        let mut r = Registers::new();
        r.pc = 0xc400;
        r.x = 0x05;
        let mut m = Memory::new();
        let instr = AddressingMode::IndirectX;

        // Immediate value of the op
        m.store(r.pc - 1, 0x3e)?;

        // Actual value in memory
        m.store16(0x0043, 0xd415)?;

        let val = instr.load_operand(&r, &m)?;
        assert_eq!(0xd415, val);
        Ok(())
    }

    #[test]
    fn test_mode_indirecty() -> IronNesResult<()> {
        let mut r = Registers::new();
        r.pc = 0xc400;
        r.y = 0x05;
        let mut m = Memory::new();
        let instr = AddressingMode::IndirectY;

        // Immediate value of the op
        m.store(r.pc - 1, 0x4c)?;

        // Actual value in memory
        m.store16(0x004c, 0xd100)?;

        let val = instr.load_operand(&r, &m)?;
        assert_eq!(0xd105, val);
        Ok(())
    }
}
