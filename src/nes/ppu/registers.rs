use crate::bitset::BiasedBitSet;
use crate::error::*;
use log::*;

use crate::nes::bus::MemoryMapped;

pub struct Registers {
    ppuctrl: BiasedBitSet,
    ppumask: BiasedBitSet,
    ppustatus: BiasedBitSet,
    oamaddr: BiasedBitSet,
    oamdata: BiasedBitSet,
    ppuscroll: BiasedBitSet,
    ppuaddr: BiasedBitSet,
    ppudata: BiasedBitSet,

    /// Proper ppu emulation requires that
    /// we have a PPU latch for some registers.
    /// It's equal to the last value present on
    /// the bus (read or write).
    /// Note that this needs to be cleared on a clock tick.
    latch: u8,
}

impl Registers {
    const PPUCTRL_ADDR: usize = 0x2000;

    pub fn new() -> Self {
        let mut ppuctrl = BiasedBitSet::default();
        ppuctrl.bias(6, 0); // Grounded on a NES
        let ppumask = BiasedBitSet::default();
        let ppustatus = BiasedBitSet::default();
        let oamdata = BiasedBitSet::default();
        let oamaddr = BiasedBitSet::default();
        let ppuscroll = BiasedBitSet::default();
        let ppuaddr = BiasedBitSet::default();
        let ppudata = BiasedBitSet::default();

        Self {
            ppuctrl,
            ppumask,
            ppustatus,
            oamdata,
            oamaddr,
            ppuscroll,
            ppuaddr,
            ppudata,
            latch: 0,
        }
    }

    pub fn reset(&mut self) {
        self.ppuctrl.store(0);
        self.ppumask.store(0);
        self.ppuscroll.store(0);
        self.ppudata.store(0);
        // dont touch others
    }

    pub fn set_vblank(&mut self, is_enabled: bool) {
        self.ppuctrl.set(7, is_enabled as u8)
    }
}

impl MemoryMapped for Registers {
    fn load(&mut self, addr: usize) -> IronNesResult<u8> {
        self.latch = match addr {
            Self::PPUCTRL_ADDR => self.ppuctrl.cast(),
            _ => {
                return Err(IronNesError::MemoryError(format!(
                    "Address not addressable: {:04x}",
                    addr
                )))
            }
        };
        Ok(self.latch)
    }

    fn store(&mut self, addr: usize, data: u8) -> IronNesResult<()> {
        self.latch = data;
        match addr {
            Self::PPUCTRL_ADDR => Ok(self.ppuctrl.store(data)),
            _ => Err(IronNesError::MemoryError(format!(
                "Address not addressable: {:04x}",
                addr
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_ppuctrl() -> IronNesResult<()> {
        let mut r = Registers::new();
        r.set_vblank(true);
        assert_eq!(0x80, r.load(Registers::PPUCTRL_ADDR)?);
        Ok(())
    }
}
