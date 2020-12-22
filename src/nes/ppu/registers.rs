use crate::bitset::BiasedBitSet;
use crate::error::*;
use log::*;

use crate::nes::bus::MemoryMapped;

pub struct Registers {
    ppuctrl: BiasedBitSet,
    ppumask: BiasedBitSet,
    ppustatus: BiasedBitSet,
    oamaddr: u8,
    oamdata: u8,
    ppuscroll: PPUScroll,
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
    const PPUMASK_ADDR: usize = 0x2001;
    const PPUSTATUS_ADDR: usize = 0x2002;
    const OAMADDR_ADDR: usize = 0x2003;
    const OAMDATA_ADDR: usize = 0x2004;
    const PPUSCROLL_ADDR: usize = 0x2005;

    pub fn new() -> Self {
        let mut ppuctrl = BiasedBitSet::default();
        ppuctrl.bias(6, 0); // Grounded on a NES
        let ppumask = BiasedBitSet::default();
        let ppustatus = BiasedBitSet::default();
        for i in 0..6 {
            // Since these values are don't cares,
            // zero them out so that reads
            // are easy to implement
            ppuctrl.bias(i, 0);
        }
        let oamdata = 0u8;
        let oamaddr = 0u8;
        let ppuscroll = PPUScroll::default();
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
        self.ppudata.store(0);
        // dont touch others
    }

    pub fn set_vblank(&mut self, is_enabled: bool) {
        self.ppuctrl.set(7, is_enabled as u8)
    }

    // TODO effective ppu reg read methods

    pub fn get_ppuscroll(&self) -> PPUScroll {
        self.ppuscroll
    }
}

#[derive(Default, Clone, Copy)]
pub struct PPUScroll {
    x: u8,
    y: u8,
}

impl PPUScroll {
    // When values are pushed into the ppuscrol reg, first it writes to
    // x then y. To simulate this, just keep pushing in values
    fn push(&mut self, v: u8) {
        self.x = self.y;
        self.y = v;
    }
}

impl MemoryMapped for Registers {
    fn load(&mut self, addr: usize) -> IronNesResult<u8> {
        self.latch = match addr {
            Self::PPUCTRL_ADDR => self.ppuctrl.cast(),
            Self::PPUMASK_ADDR => self.ppumask.cast(),
            Self::PPUSTATUS_ADDR => {
                let v = self.ppustatus.cast() | (self.latch & 0x1f);
                // Subsequent reads clear bit 7
                self.ppustatus.set(7, 0);
                v
            }
            Self::OAMADDR_ADDR => self.oamaddr,
            Self::OAMDATA_ADDR => self.oamdata,
            Self::PPUSCROLL_ADDR => {
                return Err(IronNesError::MemoryError(format!(
                    "ppuscroll is write only",
                )))
            }
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
            Self::PPUMASK_ADDR => Ok(self.ppumask.store(data)),
            Self::PPUSTATUS_ADDR => {
                Err(IronNesError::MemoryError(format!("PPUSTATUS is read only")))
            }
            Self::OAMADDR_ADDR => Ok(self.oamaddr = data),
            Self::OAMDATA_ADDR => {
                self.oamaddr = self.oamaddr.wrapping_add(1);
                Ok(self.oamdata = data)
            }
            Self::PPUSCROLL_ADDR => Ok(self.ppuscroll.push(data)),
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

    #[test]
    fn test_bus_ppustatus() -> IronNesResult<()> {
        let mut r = Registers::new();
        r.latch = 0b0101_1010u8;
        r.ppustatus.store(0xf0);
        assert_eq!(0b1111_1010u8, r.load(Registers::PPUSTATUS_ADDR)?);
        assert_eq!(
            0b0111_1010u8,
            r.load(Registers::PPUSTATUS_ADDR)?,
            "Subsequent reads should clear bit 7"
        );
        Ok(())
    }

    #[test]
    fn test_bus_ppu_oamdata() -> IronNesResult<()> {
        let mut r = Registers::new();
        r.store(Registers::OAMADDR_ADDR, 0xbe);
        assert_eq!(0xbe, r.load(Registers::OAMADDR_ADDR)?);
        r.store(Registers::OAMDATA_ADDR, 0);
        assert_eq!(
            0xbf,
            r.load(Registers::OAMADDR_ADDR)?,
            "Writing oamdata should +1 oamaddr"
        );
        Ok(())
    }

    #[test]
    fn test_bus_ppuscroll() -> IronNesResult<()> {
        let mut r = Registers::new();
        r.store(Registers::PPUSCROLL_ADDR, 0xb);
        r.store(Registers::PPUSCROLL_ADDR, 0x2);
        let scroll = r.get_ppuscroll();
        assert_eq!(0xb, scroll.x);
        assert_eq!(0x2, scroll.y);
        r.store(Registers::PPUSCROLL_ADDR, 0x7);
        let scroll = r.get_ppuscroll();
        assert_eq!(0x2, scroll.x);
        assert_eq!(0x7, scroll.y);
        Ok(())
    }
}
