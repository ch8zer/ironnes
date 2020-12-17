use crate::error::*;

use log::*;
use std::fmt;

pub type Addr = u16;

// 0x0000-0x1FFF: 0x0000 - 0x07FF are RAM           (Mirrored 4x)
// 0x2000-0x3FFF: 0x2000 - 0x2007 are PPU Regusters (Mirrored every 8 bytes)
// 0x4000-0x4013: APU registers
// 0x4014           : PPU OAM DMA
// 0x4015           : APU register
// 0x4016           : Joy1 Data (Read) and Joystick Strobe (Write)
// 0x4017           : Joy2 Data (Read) and APU thing       (Write)
// 0x4018-0x401F: APU and I/O functionality that is normally disabled
// 0x4020-0xFFFF: Cartridge ROM (may not be plugged in)
//  Typically:
//	0x6000-0x7FFF: Battery backed ROM
//	0xc000-0xFFFF: Program
// 0xFFFA-0xFFFB: NMI Vector
// 0xFFFC-0xFFFD: Reset Vector
// 0xFFFE-0xFFFF: IRQ/BRK Vector

const MEM_STACK_BEGIN: Addr = 0x0100;
const MEM_STACK_END: Addr = 0x01ff;

const MEM_RAM_BEGIN: Addr = 0x0000;
const MEM_RAM_END: Addr = 0x1fff;
const MEM_RAM_SIZE: usize = 0x800;

const MEM_PPU_BEGIN: Addr = 0x2000;
const MEM_PPU_END: Addr = 0x3fff;
const MEM_PPU_SIZE: usize = 0x8;

const MEM_REG_BEGIN: Addr = 0x4000;
const MEM_REG_END: Addr = 0x401f;
const MEM_REG_SIZE: usize = 0x20;

const MEM_BATT_ROM_BEGIN: Addr = 0x6000;
const MEM_BATT_ROM_END: Addr = 0x7fff;
const MEM_BATT_ROM_SIZE: usize = 0x2000;

const MEM_PROG_ROM_BEGIN: Addr = 0x8000;
const MEM_PROG_ROM_END: Addr = 0xffff;
const MEM_PROG_ROM_SIZE: usize = 0x8000;

pub struct Memory {
    ram: [u8; MEM_RAM_SIZE],
    ppu_reg: [u8; MEM_PPU_SIZE],
    other_reg: [u8; MEM_REG_SIZE],
    rom_batt: [u8; MEM_BATT_ROM_SIZE],
    rom_prog: [u8; MEM_PROG_ROM_SIZE],
}

// Convenience class to handle the really weird memory access patterns
enum MemoryAccess {
    RAM(usize),
    REG(usize),
    PPU(usize),
    ROMB(usize),
    ROMP(usize),
    ILLEGAL,
}

impl fmt::Display for MemoryAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryAccess::RAM(addr) => write!(f, "RAM {:04x}", addr),
            MemoryAccess::PPU(addr) => write!(f, "PPU {:04x}", addr),
            MemoryAccess::REG(addr) => write!(f, "REG {:04x}", addr),
            MemoryAccess::ROMB(addr) => write!(f, "ROMB {:04x}", addr),
            MemoryAccess::ROMP(addr) => write!(f, "ROMP {:04x}", addr),
            MemoryAccess::ILLEGAL => write!(f, "ILLEGAL"),
        }
    }
}

impl Memory {
    pub fn new() -> Self {
        Self {
            ram: [0; MEM_RAM_SIZE],
            ppu_reg: [0; MEM_PPU_SIZE],
            other_reg: [0; MEM_REG_SIZE],
            rom_batt: [0; MEM_BATT_ROM_SIZE],
            rom_prog: [0; MEM_PROG_ROM_SIZE],
        }
    }

    pub fn load_rom(&mut self, prog_rom: &Vec<u8>) -> IronNesResult<()> {
        const ONE_PAGE: usize = MEM_PROG_ROM_SIZE / 2;
        match prog_rom.len() {
            ONE_PAGE => {
                self.rom_prog[0..ONE_PAGE].clone_from_slice(&prog_rom);
                // Gets mirrored on lower bytes
                self.rom_prog[ONE_PAGE..].clone_from_slice(&prog_rom);
            }
            MEM_PROG_ROM_SIZE => {
                self.rom_prog
                    .clone_from_slice(&prog_rom[0..MEM_PROG_ROM_SIZE]);
            }
            _ => {
                error!("CARTRIDGE DOESN'T FIT");
                return Err(IronNesError::CartridgeError);
            }
        };

        Ok(())
    }

    // Since the NES has really messy memory access patterns
    fn translate_addr(addr: Addr) -> MemoryAccess {
        let a = match addr {
            MEM_RAM_BEGIN..=MEM_RAM_END => {
                MemoryAccess::RAM((addr % (MEM_RAM_SIZE as Addr)) as usize)
            }
            MEM_PPU_BEGIN..=MEM_PPU_END => {
                MemoryAccess::PPU(((addr - MEM_PPU_BEGIN) % (MEM_PPU_SIZE as Addr)) as usize)
            }
            MEM_REG_BEGIN..=MEM_REG_END => MemoryAccess::REG((addr - MEM_REG_BEGIN) as usize),
            MEM_BATT_ROM_BEGIN..=MEM_BATT_ROM_END => {
                MemoryAccess::ROMB((addr - MEM_BATT_ROM_BEGIN) as usize)
            }
            MEM_PROG_ROM_BEGIN..=MEM_PROG_ROM_END => {
                MemoryAccess::ROMP((addr - MEM_PROG_ROM_BEGIN) as usize)
            }
            _ => MemoryAccess::ILLEGAL,
        };
        trace!("Access {:04x} -> {}", addr, a);
        a
    }

    pub fn load(&self, addr: Addr) -> IronNesResult<u8> {
        let v = match Self::translate_addr(addr) {
            MemoryAccess::RAM(addr) => Ok(self.ram[addr]),
            MemoryAccess::PPU(addr) => Ok(self.ppu_reg[addr]),
            MemoryAccess::REG(addr) => Ok(self.other_reg[addr]),
            MemoryAccess::ROMB(addr) => Ok(self.rom_batt[addr]),
            MemoryAccess::ROMP(addr) => Ok(self.rom_prog[addr]),
            MemoryAccess::ILLEGAL => Err(IronNesError::MemoryError(format!(
                "Illegal load ${:04x}",
                addr
            ))),
        }?;
        trace!("mem: [{:04x}] => {:02x}", addr, v);
        Ok(v)
    }

    pub fn store(&mut self, addr: Addr, v: u8) -> IronNesResult<()> {
        trace!("mem: {:02x} => store[{:04x}]", v, addr);
        match Self::translate_addr(addr) {
            MemoryAccess::RAM(addr) => Ok(self.ram[addr] = v),
            MemoryAccess::PPU(addr) => Ok(self.ppu_reg[addr] = v),
            MemoryAccess::REG(addr) => Ok(self.other_reg[addr] = v),
            MemoryAccess::ROMB(addr) => Ok(self.rom_batt[addr] = v),
            MemoryAccess::ROMP(addr) => Ok(self.rom_prog[addr] = v),
            MemoryAccess::ILLEGAL => Err(IronNesError::MemoryError(format!(
                "Illegal store ${:04x}",
                addr
            ))),
        }
    }

    fn get_high_addr(addr: Addr) -> Addr {
        match addr {
            0..=MEM_RAM_END if ((addr & 0xff) == 0xff) => addr & 0xff00,
            _ => addr.wrapping_add(1),
        }
    }

    pub fn load16(&self, addr: Addr) -> IronNesResult<u16> {
        let high_addr = Self::get_high_addr(addr);
        let data = [self.load(addr)?, self.load(high_addr)?];
        Ok(u16::from_le_bytes(data))
    }

    pub fn store16(&mut self, addr: Addr, val: u16) -> IronNesResult<()> {
        let high_addr = Self::get_high_addr(addr);
        let bytes = val.to_le_bytes();

        self.store(addr, bytes[0])?;
        self.store(high_addr, bytes[1])
    }

    pub fn stack_push_addr(&mut self, sp: &mut Addr, addr: Addr) -> IronNesResult<()> {
        self.stack_push(sp, (addr >> 8) as u8)?;
        Ok(self.stack_push(sp, addr as u8)?)
    }

    pub fn stack_pop_addr(&mut self, sp: &mut Addr) -> IronNesResult<Addr> {
        let pcl = self.stack_pop(sp)? as Addr;
        let pch = self.stack_pop(sp)? as Addr;
        Ok((pch << 8) | pcl)
    }

    pub fn stack_push(&mut self, sp: &mut Addr, val: u8) -> IronNesResult<()> {
        if *sp == 0 {
            Err(IronNesError::MemoryError("Stack Overflow".to_string()))
        } else {
            let addr = MEM_STACK_BEGIN + *sp;
            trace!("Stack[{:04x}] PUSH {:02x}", addr, val);
            self.ram[addr as usize] = val;
            Ok(*sp = *sp - 1)
        }
    }

    pub fn stack_pop(&mut self, sp: &mut Addr) -> IronNesResult<u8> {
        if *sp == (MEM_STACK_END - MEM_STACK_BEGIN) {
            Err(IronNesError::MemoryError("Stack Underflow".to_string()))
        } else {
            *sp = *sp + 1;
            let addr = MEM_STACK_BEGIN + *sp;
            let v = self.ram[addr as usize];
            trace!("Stack[{:04x}] POP {:02x}", addr, v);
            Ok(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack() -> IronNesResult<()> {
        let mut mem = Memory::new();
        let sp0: Addr = MEM_STACK_END - MEM_STACK_BEGIN;
        let mut sp = sp0;

        let data: u64 = 0xbeef1432;

        data.to_be_bytes()
            .iter()
            .for_each(|b| mem.stack_push(&mut sp, *b).unwrap());
        assert_eq!(true, sp < sp0);

        data.to_be_bytes()
            .iter()
            .rev()
            .for_each(|b| assert_eq!(*b, mem.stack_pop(&mut sp).unwrap()));
        assert_eq!(sp0, sp);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Stack Overflow")]
    fn test_stack_overflow() {
        let mut mem = Memory::new();
        let mut sp = 0;
        mem.stack_push(&mut sp, 1).unwrap();
    }

    #[test]
    #[should_panic(expected = "Stack Underflow")]
    fn test_stack_underflow() {
        let mut mem = Memory::new();
        let mut sp = 0xff;
        mem.stack_pop(&mut sp).unwrap();
    }
}
