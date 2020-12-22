use crate::error::IronNesResult;

/**
 * Represents a memory-mapped device that lives on the NES
 * BUS (Addressable via CPU, and shared with other peripherals)
 */
pub trait MemoryMapped {
    fn load(&mut self, addr: usize) -> IronNesResult<u8>;
    fn store(&mut self, addr: usize, data: u8) -> IronNesResult<()>;
}
