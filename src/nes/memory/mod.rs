use crate::error::*;
use crate::nes::bus::Bus;

use log::*;

pub type Addr = u16;

const MEM_STACK_BEGIN: Addr = 0x0100;
const MEM_STACK_END: Addr = 0x01ff;

pub fn cpu_load(bus: &mut Bus, addr: Addr) -> IronNesResult<u8> {
    let v = bus.cpu_load(addr as usize)?;
    trace!("mem: [{:04x}] => {:02x}", addr, v);
    Ok(v)
}

pub fn cpu_store(bus: &mut Bus, addr: Addr, v: u8) -> IronNesResult<()> {
    trace!("mem: {:02x} => store[{:04x}]", v, addr);
    bus.cpu_store(addr as usize, v)
}

fn get_high_addr(addr: Addr) -> Addr {
    match addr {
        0..=0x07ff if ((addr & 0xff) == 0xff) => addr & 0xff00,
        _ => addr.wrapping_add(1),
    }
}

pub fn cpu_load16(bus: &mut Bus, addr: Addr) -> IronNesResult<u16> {
    let high_addr = get_high_addr(addr);
    let data = [cpu_load(bus, addr)?, cpu_load(bus, high_addr)?];
    Ok(u16::from_le_bytes(data))
}

pub fn cpu_store16(bus: &mut Bus, addr: Addr, val: u16) -> IronNesResult<()> {
    let high_addr = get_high_addr(addr);
    let bytes = val.to_le_bytes();

    cpu_store(bus, addr, bytes[0])?;
    cpu_store(bus, high_addr, bytes[1])
}

pub fn stack_push_addr(bus: &mut Bus, sp: &mut Addr, addr: Addr) -> IronNesResult<()> {
    stack_push(bus, sp, (addr >> 8) as u8)?;
    Ok(stack_push(bus, sp, addr as u8)?)
}

pub fn stack_pop_addr(bus: &mut Bus, sp: &mut Addr) -> IronNesResult<Addr> {
    let pcl = stack_pop(bus, sp)? as Addr;
    let pch = stack_pop(bus, sp)? as Addr;
    Ok((pch << 8) | pcl)
}

pub fn stack_push(bus: &mut Bus, sp: &mut Addr, val: u8) -> IronNesResult<()> {
    if *sp == 0 {
        Err(IronNesError::MemoryError("Stack Overflow".to_string()))
    } else {
        let addr = MEM_STACK_BEGIN + *sp;
        trace!("Stack[{:04x}] PUSH {:02x}", addr, val);
        bus.cpu_store(addr as usize, val)?;
        Ok(*sp = *sp - 1)
    }
}

pub fn stack_pop(bus: &mut Bus, sp: &mut Addr) -> IronNesResult<u8> {
    if *sp == (MEM_STACK_END - MEM_STACK_BEGIN) {
        Err(IronNesError::MemoryError("Stack Underflow".to_string()))
    } else {
        *sp = *sp + 1;
        let addr = MEM_STACK_BEGIN + *sp;
        let v = bus.cpu_load(addr as usize)?;
        trace!("Stack[{:04x}] POP {:02x}", addr, v);
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nes::bus::tests::make_bus;

    #[test]
    fn test_stack() -> IronNesResult<()> {
        let mut bus = make_bus();
        let sp0: Addr = MEM_STACK_END - MEM_STACK_BEGIN;
        let mut sp = sp0;

        let data: u64 = 0xbeef1432;

        data.to_be_bytes()
            .iter()
            .for_each(|b| stack_push(&mut bus, &mut sp, *b).unwrap());
        assert_eq!(true, sp < sp0);

        data.to_be_bytes()
            .iter()
            .rev()
            .for_each(|b| assert_eq!(*b, stack_pop(&mut bus, &mut sp).unwrap()));
        assert_eq!(sp0, sp);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Stack Overflow")]
    fn test_stack_overflow() {
        let mut bus = make_bus();
        let mut sp = 0;
        stack_push(&mut bus, &mut sp, 1).unwrap();
    }

    #[test]
    #[should_panic(expected = "Stack Underflow")]
    fn test_stack_underflow() {
        let mut bus = make_bus();
        let mut sp = 0xff;
        stack_pop(&mut bus, &mut sp).unwrap();
    }
}
