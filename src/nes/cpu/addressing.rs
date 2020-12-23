use super::register::Registers;
use crate::error::*;
use crate::nes::bus::Bus;
use crate::nes::memory::*;
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
    pub fn load_operand(&self, reg: &Registers, bus: &mut Bus) -> IronNesResult<u16> {
        // TODO better performance, don't return Addr.
        // maybe an enum or template the return?
        match self {
            AddressingMode::Accumulator => Ok(reg.a as u16),
            AddressingMode::Immediate => Ok(cpu_load(bus, reg.pc - 1)?.into()),
            AddressingMode::Absolute => cpu_load16(bus, reg.pc - 2),
            AddressingMode::AbsoluteX => {
                Ok((reg.x as u16).wrapping_add(cpu_load16(bus, reg.pc - 2)?))
            }
            AddressingMode::AbsoluteY => {
                Ok((reg.y as u16).wrapping_add(cpu_load16(bus, reg.pc - 2)?))
            }
            AddressingMode::ZeroPage => Ok(cpu_load(bus, reg.pc - 1)?.into()),
            AddressingMode::ZeroPageX => {
                Ok(((cpu_load(bus, reg.pc - 1)?.wrapping_add(reg.x)) & 0xff).into())
            }
            AddressingMode::ZeroPageY => {
                Ok(((cpu_load(bus, reg.pc - 1)?.wrapping_add(reg.y)) & 0xff).into())
            }
            AddressingMode::Indirect => {
                let imm: Addr = cpu_load16(bus, reg.pc - 2)?;
                cpu_load16(bus, imm)
            }
            AddressingMode::IndirectX => {
                let addr = cpu_load(bus, reg.pc - 1)? as Addr;
                let addr_idx = addr.wrapping_add(reg.x as Addr);
                let imm = addr_idx & 0xff;
                Ok(cpu_load16(bus, imm)?)
            }
            AddressingMode::IndirectY => {
                let imm: Addr = cpu_load(bus, reg.pc - 1)?.into();
                Ok(cpu_load16(bus, imm)?.wrapping_add(reg.y as Addr))
            }
            AddressingMode::Relative => {
                let x: Addr = cpu_load(bus, reg.pc - 1)?.into();

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
    use crate::nes::bus::tests::make_bus;

    #[test]
    fn test_mode_absolute() -> IronNesResult<()> {
        let mut bus = make_bus();
        let mut r = Registers::new();
        r.pc = 0xd005;
        r.x = 5;
        r.y = 0xff;

        cpu_store16(&mut bus, r.pc - 2, 0xc000)?;

        let instr = AddressingMode::Absolute;
        let val = instr.load_operand(&r, &mut bus)?;
        assert_eq!(0xc000, val);

        let instr = AddressingMode::AbsoluteX;
        let val = instr.load_operand(&r, &mut bus)?;
        assert_eq!(0xc005, val);

        let instr = AddressingMode::AbsoluteY;
        let val = instr.load_operand(&r, &mut bus)?;
        assert_eq!(0xc0ff, val);

        Ok(())
    }

    #[test]
    fn test_mode_zeropage() -> IronNesResult<()> {
        let mut bus = make_bus();
        let mut r = Registers::new();
        r.pc = 0xd005;
        r.x = 5;
        r.y = 0xf;

        cpu_store(&mut bus, r.pc - 1, 0xc0)?;

        let instr = AddressingMode::ZeroPage;
        let val = instr.load_operand(&r, &mut bus)?;
        assert_eq!(0xc0, val);

        let instr = AddressingMode::ZeroPageX;
        let val = instr.load_operand(&r, &mut bus)?;
        assert_eq!(0xc5, val);

        let instr = AddressingMode::ZeroPageY;
        let val = instr.load_operand(&r, &mut bus)?;
        assert_eq!(0xcf, val);

        Ok(())
    }

    #[test]
    fn test_mode_relative() -> IronNesResult<()> {
        let mut bus = make_bus();
        let mut r = Registers::new();
        r.pc = 0xc005;

        let instr = AddressingMode::Relative;

        cpu_store(&mut bus, r.pc - 1, 0x3)?;
        let val = instr.load_operand(&r, &mut bus)?;
        assert_eq!(0xc008, val);

        Ok(())
    }

    #[test]
    fn test_mode_relative_wrap() -> IronNesResult<()> {
        let mut bus = make_bus();
        let mut r = Registers::new();
        r.pc = 0xc72a + 2;

        let instr = AddressingMode::Relative;
        cpu_store(&mut bus, r.pc - 1, 0xe0)?;
        let val = instr.load_operand(&r, &mut bus)?;
        assert_eq!(0xc70c, val);

        Ok(())
    }

    #[test]
    fn test_mode_indirect() -> IronNesResult<()> {
        let mut bus = make_bus();
        let mut r = Registers::new();
        r.pc = 0xc400;
        let instr = AddressingMode::Indirect;

        // Immediate value of the op
        cpu_store16(&mut bus, r.pc - 2, 0xd15f)?;

        // Actual value in memory
        cpu_store16(&mut bus, 0xd15f, 0x3076)?;

        let val = instr.load_operand(&r, &mut bus)?;
        assert_eq!(0x3076, val);
        Ok(())
    }

    #[test]
    fn test_mode_indirectx() -> IronNesResult<()> {
        let mut bus = make_bus();
        let mut r = Registers::new();
        r.pc = 0xc400;
        r.x = 0x05;
        let instr = AddressingMode::IndirectX;

        // Immediate value of the op
        cpu_store(&mut bus, r.pc - 1, 0x3e)?;

        // Actual value in memory
        cpu_store16(&mut bus, 0x0043, 0xd415)?;

        let val = instr.load_operand(&r, &mut bus)?;
        assert_eq!(0xd415, val);
        Ok(())
    }

    #[test]
    fn test_mode_indirecty() -> IronNesResult<()> {
        let mut bus = make_bus();
        let mut r = Registers::new();
        r.pc = 0xc400;
        r.y = 0x05;
        let instr = AddressingMode::IndirectY;

        // Immediate value of the op
        cpu_store(&mut bus, r.pc - 1, 0x4c)?;

        // Actual value in memory
        cpu_store16(&mut bus, 0x004c, 0xd100)?;

        let val = instr.load_operand(&r, &mut bus)?;
        assert_eq!(0xd105, val);
        Ok(())
    }
}
