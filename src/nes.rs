pub mod cartridge;
pub mod cpu;
pub mod memory;
pub mod ppu;
use log::*;

use crate::error::*;

pub struct IronNes {
    cpu: cpu::Cpu,
    cartridge: cartridge::Cartridge,
    pub mem: memory::Memory,
}

impl IronNes {
    pub fn new() -> Self {
        info!("Starting IronNES");
        Self {
            cpu: cpu::Cpu::new(),
            cartridge: cartridge::Cartridge::default(),
            mem: memory::Memory::new(),
        }
    }

    pub fn boot(&mut self, cartridge: &str) -> IronNesResult<()> {
        info!("Loading cartridge {}", cartridge);
        let (cartridge, prog_rom, ppu_rom) = cartridge::Cartridge::load(cartridge)?;
        self.cartridge = cartridge;
        self.mem.load_rom(&prog_rom)?;

        self.reset()
    }

    pub fn reset(&mut self) -> IronNesResult<()> {
        self.cpu.reset(&self.mem)
    }

    pub fn run(&mut self) -> IronNesResult<()> {
        loop {
            self.step()?;
        }
    }

    pub fn step(&mut self) -> IronNesResult<()> {
        self.log_state()?;
        self.cpu.step(&mut self.mem)?;
        Ok(())
    }

    pub fn get_cycles(&self) -> usize {
        self.cpu.cycle
    }

    pub fn peek(&self, addr: memory::Addr) -> IronNesResult<u8> {
        self.mem.load(addr)
    }

    /**
     * CPU has a jsr method for test code, to jump to a know address
     */
    pub fn jsr(&mut self, addr: memory::Addr) -> IronNesResult<()> {
        self.cpu.jsr(addr)?;
        Ok(())
    }

    fn log_state(&self) -> IronNesResult<()> {
        info!("{}", self.cpu.log_state(&self.mem)?,);
        Ok(())
    }

    pub fn get_cpu_registers<'a>(&'a self) -> &'a cpu::Registers {
        &self.cpu.get_registers()
    }
}
