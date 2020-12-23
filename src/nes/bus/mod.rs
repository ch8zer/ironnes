use crate::error::*;

use log::*;

/**
 * Any device that is memory mapped (i.e. attached to the bus)
 * This will include: CPU memory, PPU memory & registers,
 * cartridge, controller, and mapper circuits.
 */
pub trait MemoryMapped {
    fn load(&mut self, addr: usize) -> IronNesResult<u8>;
    fn store(&mut self, addr: usize, data: u8) -> IronNesResult<()>;
}

pub type MemMappedDevice = Box<dyn MemoryMapped>;

pub struct Bus {
    cpu_zeropage: MemMappedDevice,
    cpu_oam_dma_reg: MemMappedDevice,

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

impl Bus {
    const CPU_ZEROPAGE_SIZE: usize = 0x800;
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
            cpu_oam_dma_reg: Box::new(MemoryMappedRam::new(1)),
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
    fn cpu_map<'a>(
        &'a mut self,
        addr: usize,
    ) -> IronNesResult<(usize, &'a mut Box<dyn MemoryMapped>)> {
        match addr {
            0x0000..=0x1fff => Ok((addr % Self::CPU_ZEROPAGE_SIZE, &mut self.cpu_zeropage)),
            0x2000..=0x3fff => Ok((addr % 8, &mut self.ppu_reg)),
            0x4014 => Ok((0, &mut self.cpu_oam_dma_reg)),
            0x4016..=0x4017 => Ok((addr - 0x4016, &mut self.joystick)),
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
                "Memory access to unmapped vrom {:04x}",
                addr
            ))),
        }
    }

    fn cpu_load(&mut self, addr: usize) -> IronNesResult<u8> {
        let (a, mem) = self.cpu_map(addr)?;
        trace!("bus cpu @ {:04x} => mem[{:04x}]", addr, a);
        mem.load(a)
    }

    fn cpu_store(&mut self, addr: usize, v: u8) -> IronNesResult<()> {
        let (a, mem) = self.cpu_map(addr)?;
        trace!("bus cpu @ {:04x} => mem[{:04x}]", addr, a);
        mem.store(a, v)
    }

    fn set_mapper(&mut self, mapper: Option<Box<dyn MemoryMapped>>) {
        self.cartridge_mapper = mapper
    }
}

/**
 * The simplest possible data type, just store an array
 * TODO find a way to make this memory backed via array
 */
struct MemoryMappedRam(Vec<u8>);

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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bus() -> Bus {
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
}
//fn main() {
//    let cpu = Box::new(IODevice {
//        0: [0xfe, 0xbe, 0x21, 0x43],
//    });
//    let ppu = Box::new(IODevice {
//        0: [0xfe, 0xbe, 0x21, 0x43],
//    });
//    let mut b = Bus {
//        cpu,
//        ppu,
//        mapper: None,
//    };
//
//    // Various store methods
//    println!("{}", b);
//    b.store(2, 0xff);
//    let (a, m) = b.map(0x2000);
//    m.store(a + 1, 0xff);
//    println!("{}", b);
//
//    // OAM copy
//    b.copy_out(0x2000, 0x2, 2);
//    println!("{}", b);
//
//    // Mapper
//    println!("{}", b);
//
//    b.set_mapper(Some(Box::new(IODevice { 0: [0, 0, 0, 0] })));
//
//    println!("{}", b);
//    {
//        let (a, m) = b.map(0x50);
//        m.store(a, 0x11);
//    }
//    println!("{}", b);
//
//    b.set_mapper(None);
//    println!("{}", b);
//}
