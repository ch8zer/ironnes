/*!
 * NES Emulator Core
*/
mod bus;
pub mod cartridge;
pub mod cpu;
mod memory;
mod ppu;
use log::*;

use crate::error::*;

pub struct IronNes {
    bus: bus::Bus,
    cpu: cpu::Cpu,
    ppu: ppu::Ppu,
}

impl IronNes {
    pub fn new(cartridge: &str) -> Self {
        info!("Starting IronNES");

        info!("Loading cartridge {}", cartridge);
        let (cartridge, prog_rom, ppu_rom) = cartridge::Cartridge::load(cartridge).unwrap();

        let cpu = cpu::Cpu::new();

        let ppu = ppu::Ppu::new();
        let (ppu_reg, ppu_nametables) = ppu::Ppu::alloc_mem_devices(&cartridge);

        let bus = bus::Bus::new(ppu_nametables, ppu_reg, prog_rom, ppu_rom);

        // TODO set mapper here

        Self { bus, cpu, ppu }
    }

    pub fn reset(&mut self) -> IronNesResult<()> {
        self.cpu.reset(&mut self.bus)
    }

    pub fn run(&mut self) -> IronNesResult<()> {
        loop {
            self.step()?;
        }
    }

    pub fn step(&mut self) -> IronNesResult<()> {
        self.log_state()?;
        self.cpu.step(&mut self.bus)?;
        Ok(())
    }

    pub fn get_cycles(&self) -> usize {
        self.cpu.cycle
    }

    pub fn peek(&mut self, addr: usize) -> IronNesResult<u8> {
        self.bus.cpu_load(addr)
    }

    /**
     * CPU has a jsr method for test code, to jump to a know address
     */
    pub fn jsr(&mut self, addr: usize) -> IronNesResult<()> {
        self.cpu.jsr(addr as memory::Addr)?;
        Ok(())
    }

    fn log_state(&mut self) -> IronNesResult<()> {
        info!("{}", self.cpu.log_state(&mut self.bus)?,);
        Ok(())
    }

    pub fn get_cpu_registers<'a>(&'a self) -> &'a cpu::Registers {
        &self.cpu.get_registers()
    }
}
