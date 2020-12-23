/*!
 * NMOS 6502 without DCB support.
 * Supports all NES official and unofficial opcodes.
 *
 * It is not cycle accurate, instead counts the # of cycles needed for execution.
 */

mod addressing;
mod instruction;
mod register;

use crate::error::*;
use crate::nes::bus::Bus;
use crate::nes::memory::*;
use addressing::AddressingMode;
use instruction::Instruction;
use log::*;
pub use register::{Flags, Registers};

#[derive(PartialEq)]
enum InterruptType {
    BRK,
    NMI,
    IRQ,
}

pub struct Cpu {
    pub cycle: usize,
    registers: Registers,
}

impl Cpu {
    pub const ADDR_IRQ: Addr = 0xFFFE;
    pub const ADDR_NMI: Addr = 0xFFFA;
    pub const ADDR_RESET: Addr = 0xFFFC;

    pub fn new() -> Self {
        Self {
            cycle: 0,
            registers: Registers::new(),
        }
    }

    pub fn get_registers<'a>(&'a self) -> &'a register::Registers {
        &self.registers
    }

    pub fn reset(&mut self, bus: &mut Bus) -> IronNesResult<()> {
        self.cycle = 0;

        self.registers = register::Registers::new();
        self.registers.pc = cpu_load16(bus, Self::ADDR_RESET)?;

        warn!("IronNES PC reset to RESET VEC {:04x}", self.registers.pc);
        Ok(())
    }

    /**
     * Performs a single step of CPU, executing a whole instruction (for now).
     * Instruction implementation/reference from: http://nesdev.com/6502.txt
     */
    pub fn step(&mut self, bus: &mut Bus) -> IronNesResult<Instruction> {
        let opcode = cpu_load(bus, self.registers.pc)?;

        let instr = Instruction::lookup(opcode);
        self.cycle += instr.cycles;

        // Increment the program counter
        self.registers.pc = self.registers.pc.wrapping_add(instr.bytes.into());

        // Generated jump-table to make the code less verbose
        include!(concat!(env!("OUT_DIR"), "/instr_jumptable.rs"))?;

        Ok(instr)
    }

    pub fn calc_page_cross_penalty(addr1: Addr, addr2: Addr) -> usize {
        ((addr1 & 0xff00) != (addr2 & 0xff00)) as usize
    }

    /// Used to advanced the CPU to a future instruction
    pub fn jsr(&mut self, addr: Addr) -> IronNesResult<Instruction> {
        let instr = Instruction::lookup(0x20);
        self.cycle += instr.cycles;
        self.registers.pc = addr;
        Ok(instr)
    }

    // Interrupts can happen on NON-brk instructions...
    fn interrupt(&mut self, bus: &mut Bus, t: InterruptType) -> IronNesResult<()> {
        if self.registers.get_flag(Flags::I) && t == InterruptType::IRQ {
            warn!("IRQ not allowed when I==1.");
            return Ok(());
        }

        let pc = match t {
            InterruptType::BRK => self.registers.pc + 1,
            _ => self.registers.pc,
        };

        stack_push_addr(bus, &mut self.registers.sp, pc)?;
        self.registers.set_flag(Flags::B, t == InterruptType::BRK);
        let status = self.registers.get_status();
        stack_push(bus, &mut self.registers.sp, status)?;
        self.registers.set_flag(Flags::I, true);

        let addr: Addr = match t {
            InterruptType::BRK => Self::ADDR_IRQ,
            InterruptType::IRQ => Self::ADDR_IRQ,
            InterruptType::NMI => Self::ADDR_NMI,
        };

        Ok(self.registers.pc = cpu_load16(bus, addr)?)
    }

    pub fn log_state(&self, bus: &mut Bus) -> IronNesResult<String> {
        let opcode = cpu_load(bus, self.registers.pc)?;
        let instr = Instruction::lookup(opcode);

        Ok(format!(
            "{:04x} {:28} {} CYC {}",
            self.registers.pc,
            instr.print(self.registers.pc - (instr.bytes as u16), bus),
            self.registers,
            self.cycle
        ))
    }
}

fn pay_for_page_cross(cpu: &mut Cpu, instr: &Instruction, addr: Addr) -> IronNesResult<()> {
    if instr.can_cross_page {
        let src_addr = match instr.addr_mode {
            AddressingMode::Relative => cpu.registers.pc,
            AddressingMode::AbsoluteX => addr.wrapping_sub(cpu.registers.x as Addr),
            AddressingMode::AbsoluteY | AddressingMode::IndirectY => {
                addr.wrapping_sub(cpu.registers.y as Addr)
            }
            _ => addr,
        };
        let penalty = Cpu::calc_page_cross_penalty(src_addr, addr);
        trace!(
            "Paying {} cycles for page cross penalty [${:04x} -> ${:04x}]",
            penalty,
            src_addr,
            addr
        );
        cpu.cycle += penalty;
    }

    Ok(())
}

/// Used in case our addressing mode requires and extra lookup to fetch the operand.
fn fetch_operand(
    cpu: &mut Cpu,
    instr: &Instruction,
    bus: &mut Bus,

    addr: Addr,
) -> IronNesResult<u8> {
    pay_for_page_cross(cpu, instr, addr)?;
    match instr.addr_mode {
        AddressingMode::Absolute
        | AddressingMode::AbsoluteX
        | AddressingMode::AbsoluteY
        | AddressingMode::ZeroPage
        | AddressingMode::ZeroPageX
        | AddressingMode::ZeroPageY
        | AddressingMode::Indirect
        | AddressingMode::IndirectX
        | AddressingMode::IndirectY => cpu_load(bus, addr),
        _ => Ok(addr as u8),
    }
}

#[allow(unused_variables)]
fn nop_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    if instr.addr_mode == AddressingMode::AbsoluteX {
        let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
        pay_for_page_cross(cpu, instr, addr)?;
    }
    Ok(())
}

#[allow(unused_variables)]
fn brk_execute(cpu: &mut Cpu, _instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    cpu.interrupt(bus, InterruptType::BRK)
}

#[allow(unused_variables)]
fn cmp_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    do_cmp(cpu, &instr, bus, cpu.registers.a)
}

#[allow(unused_variables)]
fn cpx_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    do_cmp(cpu, &instr, bus, cpu.registers.x)
}

#[allow(unused_variables)]
fn cpy_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    do_cmp(cpu, &instr, bus, cpu.registers.y)
}

#[allow(unused_variables)]
fn bcc_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    br_execute(cpu, &instr, bus, Flags::C, false)
}

#[allow(unused_variables)]
fn bcs_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    br_execute(cpu, &instr, bus, Flags::C, true)
}

#[allow(unused_variables)]
fn beq_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    br_execute(cpu, &instr, bus, Flags::Z, true)
}

#[allow(unused_variables)]
fn bmi_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    br_execute(cpu, &instr, bus, Flags::N, true)
}

#[allow(unused_variables)]
fn bne_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    br_execute(cpu, &instr, bus, Flags::Z, false)
}

#[allow(unused_variables)]
fn bpl_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    br_execute(cpu, &instr, bus, Flags::N, false)
}

#[allow(unused_variables)]
fn bvc_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    br_execute(cpu, &instr, bus, Flags::V, false)
}

#[allow(unused_variables)]
fn bvs_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    br_execute(cpu, &instr, bus, Flags::V, true)
}

#[allow(unused_variables)]
fn rti_execute(cpu: &mut Cpu, _instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let orig = cpu.registers.get_status() & 0b00110000;
    let v = stack_pop(bus, &mut cpu.registers.sp)? & 0b11001111;
    let v = v | orig;
    cpu.registers.set_status(v);

    Ok(cpu.registers.pc = stack_pop_addr(bus, &mut cpu.registers.sp)?)
}

#[allow(unused_variables)]
fn adc_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let s = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let s = fetch_operand(cpu, instr, bus, s)?;
    let a = cpu.registers.a;
    let c = cpu.registers.get_flag(Flags::C) as u16;

    if cpu.registers.get_flag(Flags::D) {
        error!("DCB not supported on NES, using int math");
    }

    let sum: u16 = (a as u16) + (s as u16) + c;
    cpu.registers.a = sum as u8;

    let v = {
        let x = a as u16;
        let y = s as u16;

        let l = ((x ^ sum) & 0x80) != 0;
        let r = ((x ^ y) & 0x80) != 0;
        !r & l
    };

    cpu.registers.set_z(sum & 0xff);
    cpu.registers.set_flag(Flags::C, (sum & 0xFF00) != 0);
    cpu.registers.set_flag(Flags::V, v);
    cpu.registers.set_n(sum);

    Ok(())
}

#[allow(unused_variables)]
fn sbc_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let s = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let s = fetch_operand(cpu, instr, bus, s)?;

    let a = cpu.registers.a;
    let c = !cpu.registers.get_flag(Flags::C) as i16;

    if cpu.registers.get_flag(Flags::D) {
        error!("DCB not supported on NES, using int math");
    }

    let sum = ((a as i16) - (s as i16) - c) as u16;
    cpu.registers.a = sum as u8;

    let v = {
        let x = a as u16;
        let y = s as u16;

        let l = ((x ^ sum) & 0x80) != 0;
        let r = ((x ^ y) & 0x80) != 0;
        r & l
    };
    cpu.registers.set_z(sum & 0xff);
    cpu.registers.set_flag(Flags::C, sum < 0x100);
    cpu.registers.set_flag(Flags::V, v);
    cpu.registers.set_n(sum);

    Ok(())
}

fn increment_helper(src: u8, amt: i16, reg: &mut Registers) -> IronNesResult<u8> {
    let src = src as i16;
    let val: i16 = (src + amt) & 0xff;
    let val = val as u16;
    reg.set_z(val);
    reg.set_n(val);
    Ok(val as u8)
}

#[allow(unused_variables)]
fn inc_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let s = fetch_operand(cpu, instr, bus, addr)?;
    let s = increment_helper(s, 1, &mut cpu.registers)?;
    cpu_store(bus, addr, s)
}

#[allow(unused_variables)]
fn inx_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    Ok(cpu.registers.x = increment_helper(cpu.registers.x, 1, &mut cpu.registers)?)
}

#[allow(unused_variables)]
fn iny_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    Ok(cpu.registers.y = increment_helper(cpu.registers.y, 1, &mut cpu.registers)?)
}

#[allow(unused_variables)]
fn dec_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let s = fetch_operand(cpu, instr, bus, addr)?;
    let s = increment_helper(s, -1, &mut cpu.registers)?;
    cpu_store(bus, addr, s)
}

#[allow(unused_variables)]
fn dex_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    Ok(cpu.registers.x = increment_helper(cpu.registers.x, -1, &mut cpu.registers)?)
}

#[allow(unused_variables)]
fn dey_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    Ok(cpu.registers.y = increment_helper(cpu.registers.y, -1, &mut cpu.registers)?)
}

#[allow(unused_variables)]
fn dcp_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    dec_execute(cpu, instr, bus)?;
    do_cmp(cpu, instr, bus, cpu.registers.a)
}

#[allow(unused_variables)]
fn isc_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    inc_execute(cpu, instr, bus)?;
    sbc_execute(cpu, instr, bus)
}

#[allow(unused_variables)]
fn and_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let s = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let s = fetch_operand(cpu, instr, bus, s)?;
    cpu.registers.a &= s;
    cpu.registers.set_n(cpu.registers.a.into());
    cpu.registers.set_z(cpu.registers.a.into());

    Ok(())
}

#[allow(unused_variables)]
fn do_cmp(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus, src: u8) -> IronNesResult<()> {
    let s = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let s = fetch_operand(cpu, instr, bus, s)?;

    let sum = (src as i16) - (s as i16);
    cpu.registers.set_flag(Flags::C, (sum as u16) < 0x100);
    cpu.registers.set_flag(Flags::N, (sum & 0x80) != 0);
    cpu.registers.set_z(sum as u16);

    Ok(())
}

#[allow(unused_variables)]
fn ora_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let s = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let s = fetch_operand(cpu, instr, bus, s)?;
    let a = cpu.registers.a;
    cpu.registers.a = a | s;
    cpu.registers.set_n(cpu.registers.a.into());
    cpu.registers.set_z(cpu.registers.a.into());

    Ok(())
}

#[allow(unused_variables)]
fn eor_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let s = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let s = fetch_operand(cpu, instr, bus, s)?;
    let a = cpu.registers.a;
    cpu.registers.a = a ^ s;
    cpu.registers.set_n(cpu.registers.a.into());
    cpu.registers.set_z(cpu.registers.a.into());

    Ok(())
}

#[allow(unused_variables)]
fn bit_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let s = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let s = cpu_load(bus, s)?;

    cpu.registers.set_flag(Flags::Z, (cpu.registers.a & s) == 0);
    cpu.registers.set_flag(Flags::V, (s & 0x40) != 0);
    cpu.registers.set_flag(Flags::N, (s & 0x80) != 0);

    Ok(())
}

#[allow(unused_variables)]
fn br_execute(
    cpu: &mut Cpu,
    instr: &Instruction,

    bus: &mut Bus,
    flag: Flags,
    state: bool,
) -> IronNesResult<()> {
    if state == cpu.registers.get_flag(flag) {
        let dest = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
        // Add one for taking the br
        cpu.cycle += 1;
        // Add another for crossing the page boundary
        pay_for_page_cross(cpu, instr, dest)?;
        cpu.registers.pc = dest;
        trace!("Taking branch to {:04x}", cpu.registers.pc);
    }
    Ok(())
}

#[allow(unused_variables)]
fn setp_execute(reg: &mut Registers, flag: Flags, state: bool) -> IronNesResult<()> {
    reg.set_flag(flag, state);
    Ok(())
}

#[allow(unused_variables)]
fn sec_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    setp_execute(&mut cpu.registers, Flags::C, true)
}

#[allow(unused_variables)]
fn sed_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    setp_execute(&mut cpu.registers, Flags::D, true)
}

#[allow(unused_variables)]
fn sei_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    setp_execute(&mut cpu.registers, Flags::I, true)
}

#[allow(unused_variables)]
fn clc_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    setp_execute(&mut cpu.registers, Flags::C, false)
}

#[allow(unused_variables)]
fn cld_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    setp_execute(&mut cpu.registers, Flags::D, false)
}

#[allow(unused_variables)]
fn cli_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    setp_execute(&mut cpu.registers, Flags::I, false)
}

#[allow(unused_variables)]
fn clv_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    setp_execute(&mut cpu.registers, Flags::V, false)
}

#[allow(unused_variables)]
fn jsr_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    stack_push_addr(bus, &mut cpu.registers.sp, cpu.registers.pc - 1)?;
    Ok(cpu.registers.pc = instr.addr_mode.load_operand(&mut cpu.registers, bus)?)
}

#[allow(unused_variables)]
fn jmp_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    cpu.registers.pc = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    Ok(())
}

#[allow(unused_variables)]
fn ld_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<u8> {
    let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let v = fetch_operand(cpu, instr, bus, addr)?;
    cpu.registers.set_n(v.into());
    cpu.registers.set_z(v.into());
    Ok(v)
}

#[allow(unused_variables)]
fn lax_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    cpu.registers.a = ld_execute(cpu, instr, bus)?;
    Ok(cpu.registers.x = cpu.registers.a)
}

#[allow(unused_variables)]
fn lda_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    Ok(cpu.registers.a = ld_execute(cpu, instr, bus)?)
}

#[allow(unused_variables)]
fn ldx_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    Ok(cpu.registers.x = ld_execute(cpu, instr, bus)?)
}

#[allow(unused_variables)]
fn ldy_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    Ok(cpu.registers.y = ld_execute(cpu, instr, bus)?)
}

#[allow(unused_variables)]
fn asl_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let v = fetch_operand(cpu, instr, bus, addr)?;

    cpu.registers.set_flag(Flags::C, (v & 0x80) != 0);
    let v = v << 1;
    cpu.registers.set_n(v.into());
    cpu.registers.set_z(v.into());

    match instr.addr_mode {
        AddressingMode::Accumulator => cpu.registers.a = v,
        _ => cpu_store(bus, addr, v)?,
    };

    Ok(())
}

#[allow(unused_variables)]
fn lsr_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let v = fetch_operand(cpu, instr, bus, addr)?;

    cpu.registers.set_flag(Flags::C, (v & 1) != 0);
    let v = v >> 1;
    cpu.registers.set_n(v.into());
    cpu.registers.set_z(v.into());

    match instr.addr_mode {
        AddressingMode::Accumulator => cpu.registers.a = v,
        _ => cpu_store(bus, addr, v)?,
    };

    Ok(())
}

#[allow(unused_variables)]
fn rol_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let v = fetch_operand(cpu, instr, bus, addr)?;

    let v = v as u16;
    let v = (v << 1) | (cpu.registers.get_flag(Flags::C) as u16);

    cpu.registers.set_flag(Flags::C, v > 0xff);
    let v = v as u8;
    cpu.registers.set_n(v.into());
    cpu.registers.set_z(v.into());

    match instr.addr_mode {
        AddressingMode::Accumulator => cpu.registers.a = v,
        _ => cpu_store(bus, addr, v)?,
    };

    Ok(())
}

#[allow(unused_variables)]
fn ror_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let v = fetch_operand(cpu, instr, bus, addr)?;

    let c = match cpu.registers.get_flag(Flags::C) {
        true => 0x100,
        _ => 0,
    };

    cpu.registers.set_flag(Flags::C, (v & 1) != 0);

    let v = (v as u16 | c) >> 1;
    let v = v as u8;

    cpu.registers.set_n(v.into());
    cpu.registers.set_z(v.into());

    match instr.addr_mode {
        AddressingMode::Accumulator => cpu.registers.a = v,
        _ => cpu_store(bus, addr, v)?,
    };

    Ok(())
}

#[allow(unused_variables)]
fn pha_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    stack_push(bus, &mut cpu.registers.sp, cpu.registers.a)
}

#[allow(unused_variables)]
fn php_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let v = cpu.registers.get_status() | 0b00110000;
    stack_push(bus, &mut cpu.registers.sp, v)
}

#[allow(unused_variables)]
fn pla_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    cpu.registers.a = stack_pop(bus, &mut cpu.registers.sp)?;
    cpu.registers.set_n(cpu.registers.a.into());
    cpu.registers.set_z(cpu.registers.a.into());
    Ok(())
}

#[allow(unused_variables)]
fn plp_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let orig = cpu.registers.get_status() & 0b00110000;
    let v = stack_pop(bus, &mut cpu.registers.sp)? & 0b11001111;
    let v = v | orig;
    Ok(cpu.registers.set_status(v))
}

#[allow(unused_variables)]
fn rts_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    Ok(cpu.registers.pc = 1 + stack_pop_addr(bus, &mut cpu.registers.sp)?)
}

#[allow(unused_variables)]
fn sax_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    let v = cpu.registers.a & cpu.registers.x;
    cpu_store(bus, addr, v)
}

#[allow(unused_variables)]
fn sta_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    cpu_store(bus, addr, cpu.registers.a)
}

#[allow(unused_variables)]
fn stx_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    cpu_store(bus, addr, cpu.registers.x)
}

#[allow(unused_variables)]
fn sty_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let addr = instr.addr_mode.load_operand(&mut cpu.registers, bus)?;
    cpu_store(bus, addr, cpu.registers.y)
}

#[allow(unused_variables)]
fn tax_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let src = cpu.registers.a;
    cpu.registers.set_n(src.into());
    cpu.registers.set_z(src.into());
    Ok(cpu.registers.x = src)
}

#[allow(unused_variables)]
fn tay_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let src = cpu.registers.a;
    cpu.registers.set_n(src.into());
    cpu.registers.set_z(src.into());
    Ok(cpu.registers.y = src)
}

#[allow(unused_variables)]
fn tsx_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let src = cpu.registers.sp as u8;
    cpu.registers.set_n(src.into());
    cpu.registers.set_z(src.into());
    Ok(cpu.registers.x = src)
}

#[allow(unused_variables)]
fn txa_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let src = cpu.registers.x;
    cpu.registers.set_n(src.into());
    cpu.registers.set_z(src.into());
    Ok(cpu.registers.a = src)
}

#[allow(unused_variables)]
fn txs_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    Ok(cpu.registers.sp = cpu.registers.x.into())
}

#[allow(unused_variables)]
fn tya_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    let src = cpu.registers.y;
    cpu.registers.set_n(src.into());
    cpu.registers.set_z(src.into());
    Ok(cpu.registers.a = src)
}

#[allow(unused_variables)]
fn slo_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    asl_execute(cpu, instr, bus)?;
    ora_execute(cpu, instr, bus)
}

#[allow(unused_variables)]
fn rla_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    rol_execute(cpu, instr, bus)?;
    and_execute(cpu, instr, bus)
}

#[allow(unused_variables)]
fn rra_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    ror_execute(cpu, instr, bus)?;
    adc_execute(cpu, instr, bus)
}

#[allow(unused_variables)]
fn sre_execute(cpu: &mut Cpu, instr: &Instruction, bus: &mut Bus) -> IronNesResult<()> {
    lsr_execute(cpu, instr, bus)?;
    eor_execute(cpu, instr, bus)
}
