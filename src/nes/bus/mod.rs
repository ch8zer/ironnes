pub mod memory_mapped;

use crate::error::*;
use memory_mapped::{MemMappedDevice, MemoryMappedRam};

use log::*;

/**
 * The bus holds all memory mapped devices for all computational units.
 */
#[allow(dead_code)] // TODO remove
pub struct Bus {
    cpu_zeropage: MemMappedDevice,
    cpu_reg: MemMappedDevice,

    ppu_reg: MemMappedDevice,
    ppu_nametables: MemMappedDevice,
    ppu_palette_ram: MemMappedDevice,

    oam: MemMappedDevice,

    joystick: MemMappedDevice,

    cartridge_rom: MemMappedDevice,
    cartridge_rom_offset: usize,
    cartridge_vram: MemMappedDevice,
    cartridge_mapper: Option<MemMappedDevice>,
}

#[allow(dead_code)] // TODO remove
impl Bus {
    const CPU_ZEROPAGE_SIZE: usize = 0x800;
    const CPU_REG_SIZE: usize = 0x18;
    const OAM_SIZE: usize = 256;
    const PPU_PALETTE_RAM_SIZE: usize = 0x20;
    const NUM_JOYSTICK: usize = 2;

    const PAGE_SIZE: usize = 0x4000;

    pub fn new(
        ppu_nametables: MemMappedDevice,
        ppu_reg: MemMappedDevice,
        cartridge_rom: Vec<u8>,
        cartridge_vram: Vec<u8>,
    ) -> Self {
        let num_pages = cartridge_rom.len() / Self::PAGE_SIZE;
        let cartridge_rom_offset = match num_pages {
            1 => 0xc000,
            2 => 0x8000,
            _ => panic!(
                "cartridge has an unsupported number of rom pages {}",
                num_pages
            ),
        };

        Self {
            cpu_zeropage: Box::new(MemoryMappedRam::new(Self::CPU_ZEROPAGE_SIZE)),
            cpu_reg: Box::new(MemoryMappedRam::new(Self::CPU_REG_SIZE)),
            ppu_reg,
            ppu_nametables,
            ppu_palette_ram: Box::new(MemoryMappedRam::new(Self::PPU_PALETTE_RAM_SIZE)),
            oam: Box::new(MemoryMappedRam::new(Self::OAM_SIZE)),
            joystick: Box::new(MemoryMappedRam::new(Self::NUM_JOYSTICK)),
            cartridge_rom: Box::new(MemoryMappedRam::from_vec(cartridge_rom)),
            cartridge_rom_offset,
            cartridge_vram: Box::new(MemoryMappedRam::from_vec(cartridge_vram)),
            cartridge_mapper: None,
        }
    }

    /**
     * Given an address we are interested in, map that to a tuple
     * of (addr2, MemMappedDevice) where addr2 is the translated address
     * that your MemMappedDevice understands
     *
     * FIXME the values are hardcoded here ONCE to cut down on the # of lines
     * of code I need to write. Is this the right thing to do? Probably not, but
     * it keeps the file short.
     */
    fn cpu_map<'a>(&'a mut self, addr: usize) -> IronNesResult<(usize, &'a mut MemMappedDevice)> {
        match addr {
            0x0000..=0x1fff => Ok((addr % Self::CPU_ZEROPAGE_SIZE, &mut self.cpu_zeropage)),
            0x2000..=0x3fff => Ok((addr % 8, &mut self.ppu_reg)),
            0x4000..=0x4017 => Ok((addr - 0x4000, &mut self.cpu_reg)),
            0x8000..=0xffff if self.cartridge_rom_offset == 0x8000 => {
                Ok((addr - self.cartridge_rom_offset, &mut self.cartridge_rom))
            }
            0x8000..=0xffff if self.cartridge_rom_offset == 0xc000 => {
                if addr < self.cartridge_rom_offset {
                    return match &mut self.cartridge_mapper {
                        Some(m) => Ok((addr - 0x8000, m)),
                        None => Err(IronNesError::MemoryError(format!("No mapper inserted"))),
                    };
                }
                Ok((addr - self.cartridge_rom_offset, &mut self.cartridge_rom))
            }
            _ => Err(IronNesError::MemoryError(format!(
                "Memory access to unmapped cpu addr {:04x}",
                addr
            ))),
        }
    }

    /**
     * Given an address we are interested in, map that to a tuple
     * of (addr2, MemMappedDevice) where addr2 is the translated address
     * that your MemMappedDevice understands
     *
     * FIXME the values are hardcoded here ONCE to cut down on the # of lines
     * of code I need to write. Is this the right thing to do? Probably not, but
     * it keeps the file short.
     */
    fn ppu_map<'a>(&'a mut self, addr: usize) -> IronNesResult<(usize, &'a mut MemMappedDevice)> {
        match addr {
            0x0000..=0x1fff => Ok((addr, &mut self.cartridge_vram)),
            0x2000..=0x3eff => Ok((addr & 0x0fff, &mut self.ppu_reg)),
            0x3f00..=0x3fff => Ok((addr & 0x1f, &mut self.ppu_palette_ram)),
            _ => Err(IronNesError::MemoryError(format!(
                "Memory access to unmapped ppu addr {:04x}",
                addr
            ))),
        }
    }

    pub fn cpu_load(&mut self, addr: usize) -> IronNesResult<u8> {
        let (a, mem) = self.cpu_map(addr)?;
        trace!("bus cpu @ {:04x} => mem[{:04x}]", addr, a);
        mem.load(a)
    }

    pub fn cpu_store(&mut self, addr: usize, v: u8) -> IronNesResult<()> {
        let (a, mem) = self.cpu_map(addr)?;
        trace!("bus cpu @ {:04x} => mem[{:04x}]", addr, a);
        mem.store(a, v)
    }

    pub fn ppu_get_reg<'a>(&'a mut self) -> &'a mut MemMappedDevice {
        &mut self.ppu_reg
    }

    fn set_mapper(&mut self, mapper: Option<MemMappedDevice>) {
        self.cartridge_mapper = mapper
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn make_bus() -> Bus {
        let ppu_nametables = Box::new(MemoryMappedRam::new(0));
        let ppu_reg = Box::new(MemoryMappedRam::new(8));
        let cartridge_rom = vec![0; Bus::PAGE_SIZE];
        let cartridge_vram = vec![0; Bus::PAGE_SIZE];

        Bus::new(ppu_nametables, ppu_reg, cartridge_rom, cartridge_vram)
    }

    #[test]
    fn test_bus_cpu_zeropage() {
        let mut bus = make_bus();

        // how big a zero page is
        const ZP: usize = 0x800; // how big is ZP
        const NP: usize = 4; // # of mirrors

        let test_inputs = vec![0x13, 0xbe, 0x69, 0x42];

        test_inputs
            .iter()
            .enumerate()
            .for_each(|(i, v)| bus.cpu_store(v + (ZP * (i % NP)), *v as u8).unwrap());

        test_inputs.iter().for_each(|v| {
            for i in 0..NP {
                let addr = v + (ZP * i);
                let x = bus.cpu_load(addr).unwrap();
                assert_eq!((*v as u8), x,);
            }
        });
    }

    #[test]
    fn test_bus_ppu_reg() {
        let mut bus = make_bus();

        // Write to registers # 3,4
        bus.cpu_store(0x2003, 1).unwrap();
        bus.cpu_store(0x3404 + 8, 2).unwrap();

        // Mirroring of indicies
        let x = bus.cpu_load(0x2003 + 0x1168).unwrap();
        assert_eq!(1, x);
        let x = bus.cpu_load(0x2004).unwrap();
        assert_eq!(2, x);

        // PPU shared with cpu
        {
            let reg = bus.ppu_get_reg();
            let x = reg.load(3).unwrap();
            assert_eq!(1, x);
            let x = reg.load(4).unwrap();
            assert_eq!(2, x);
            reg.store(3, 3).unwrap();
        }

        let x = bus.cpu_load(0x2003 + 0x1008).unwrap();
        assert_eq!(3, x);
    }
}
