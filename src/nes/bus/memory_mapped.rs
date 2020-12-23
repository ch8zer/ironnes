use crate::error::*;

pub type MemMappedDevice = Box<dyn MemoryMapped>;

/**
 * Any device that is memory mapped (i.e. attached to the bus)
 * This will include: CPU memory, PPU memory & registers,
 * cartridge, controller, and mapper circuits.
 */
pub trait MemoryMapped {
    fn load(&mut self, addr: usize) -> IronNesResult<u8>;
    fn store(&mut self, addr: usize, data: u8) -> IronNesResult<()>;

    fn get_ref<'a>(&'a self) -> Option<&'a [u8]>;
    fn get_mut_ref<'a>(&'a mut self) -> Option<&'a mut [u8]>;
}

/**
 * The simplest possible data type, just store an array
 * TODO find a way to make this memory backed via array
 */
pub struct MemoryMappedRam(Vec<u8>);

impl MemoryMappedRam {
    pub fn new(size: usize) -> Self {
        Self { 0: vec![0; size] }
    }

    pub fn from_vec(vals: Vec<u8>) -> Self {
        Self { 0: vals }
    }
}

impl MemoryMapped for MemoryMappedRam {
    fn load(&mut self, addr: usize) -> IronNesResult<u8> {
        if addr > self.0.len() {
            return Err(IronNesError::MemoryError(format!(
                "load out of range ${:04x}",
                addr
            )));
        }

        Ok(self.0[addr])
    }

    fn store(&mut self, addr: usize, data: u8) -> IronNesResult<()> {
        if addr > self.0.len() {
            return Err(IronNesError::MemoryError(format!(
                "store out of range ${:04x}",
                addr
            )));
        }

        Ok(self.0[addr] = data)
    }

    fn get_ref<'a>(&'a self) -> Option<&'a [u8]> {
        Some(&self.0)
    }

    fn get_mut_ref<'a>(&'a mut self) -> Option<&'a mut [u8]> {
        Some(&mut self.0)
    }
}
