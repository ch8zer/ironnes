use crate::error::*;

use log::*;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Clone, Default)]
pub struct Cartridge {
    num_prog_rom: usize,
    num_ppu_vrom: usize,
    num_ram: usize,
    pub mirror: MirrorDirection,
    pub has_battery: bool,
    pub has_trainer: bool,
    pub mapper: u8,
    pub region: CartridgeRegion,
}

/**
 * Cartridge Layout (.nes file)
 * Byte   | Contents
 * -------|-------------------------------------------------------------------
 * 0-3    | String "NES^Z" used to recognize .NES files.
 * 4      | Number of 16kB ROM banks.
 * 5      | Number of 8kB VROM banks.
 * 6      | bit 0     1 for vertical mirroring, 0 for horizontal mirroring.
 *        | bit 1     1 for battery-backed RAM at $6000-$7FFF.
 *        | bit 2     1 for a 512-byte trainer at $7000-$71FF.
 *        | TODO bit 3     1 for a four-screen VRAM layout.
 *        | bit 4-7   Four lower bits of ROM Mapper Type.
 * 7      | TODO bit 0     1 for VS-System cartridges.
 *        | bit 1-3   Reserved, must be zeroes!
 *        | bit 4-7   Four higher bits of ROM Mapper Type.
 * 8      | Number of 8kB RAM banks. For compatibility with the previous
 *        | versions of the .NES format, assume 1x8kB RAM page when this
 *        | byte is zero.
 * 9      | bit 0     1 for PAL cartridges, otherwise assume NTSC.
 *        | bit 1-7   Reserved, must be zeroes!
 * 10-15  | Reserved, must be zeroes!
 * 16-... | DATA - ROM banks, in ascending order. If a trainer is present, its
 *        | 512 bytes precede the ROM bank contents.
 * ...-EOF| PROG - VROM banks, in ascending order.
 */
impl Cartridge {
    pub const CARTRIDGE_HEADER: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];
    pub const NES_FILE_HEADER_SIZE: usize = 16;

    pub const CHIP_SIZE_PROG: usize = 0x4000;
    const CHIP_SIZE_PPU: usize = 0x2000;
    const CHIP_SIZE_RAM: usize = 0x2000;

    /**
     * Parses the cartridge file and returns a tuple of (Cartridge, prog_bytes, ppu_bytes)
     */
    pub fn load(cartridge_file: &str) -> IronNesResult<(Self, Vec<u8>, Vec<u8>)> {
        if !Path::new(cartridge_file).exists() {
            error!(
                "Catridge '{}' can not be found on filesystem",
                cartridge_file
            );
            return Err(IronNesError::CartridgeError);
        }

        let mut f = File::open(cartridge_file)?;

        let mut header = vec![0u8; Self::NES_FILE_HEADER_SIZE];
        f.read(&mut header)?;

        let cartridge = Self::from_header(&header)?;
        warn!("Read Cartridge: {}", cartridge);

        let mut prog_rom = vec![0u8; cartridge.get_prog_size()];
        let mut ppu_vrom = vec![0u8; cartridge.get_ppu_size()];

        f.read(&mut prog_rom)?;
        f.read(&mut ppu_vrom)?;

        Ok((cartridge, prog_rom, ppu_vrom))
    }

    pub fn from_header(cartridge: &[u8]) -> IronNesResult<Self> {
        Self::cartridge_header_check(cartridge)?;

        if (cartridge[7] & 0b1110u8) != 0 || (cartridge[9] & 0b11111110u8) != 0 {
            error!("Catridge 0 sections invalid");
            return Err(IronNesError::CartridgeError);
        }

        let mut c = Cartridge::default();

        c.num_prog_rom = cartridge[4] as usize;
        trace!("Cartridge has {} prog chips", c.num_prog_rom);
        c.num_ppu_vrom = cartridge[5] as usize;
        trace!("Cartridge has {} ppu chips", c.num_ppu_vrom);
        c.num_ram = cartridge[8] as usize;
        trace!("Cartridge has {} ram chips", c.num_ram);

        let has_4s = (cartridge[6] & 0b1000) != 0;
        let has_vert = (cartridge[6] & 1) != 0;

        c.mirror = match (has_4s, has_vert) {
            (true, _) => MirrorDirection::FourScreen,
            (false, true) => MirrorDirection::Vertical,
            (false, false) => MirrorDirection::Horizontal,
        };

        c.has_battery = (cartridge[6] & 0b10) > 0;
        c.has_trainer = (cartridge[6] & 0b100) > 0;

        c.mapper = (cartridge[6] & 0xf0) >> 4;
        c.mapper = c.mapper & cartridge[7] & 0xf0;

        if c.mapper != 0 {
            error!(
                "Emulator does not support mappers. Requested: {}",
                which_mapper(c.mapper)
            );
            return Err(IronNesError::CartridgeError);
        }

        c.region = match cartridge[9] & 1 {
            1 => CartridgeRegion::PAL,
            _ => CartridgeRegion::NTSC,
        };

        Ok(c)
    }

    fn cartridge_header_check(cartridge: &[u8]) -> IronNesResult<()> {
        if cartridge[0..Self::CARTRIDGE_HEADER.len()] != Self::CARTRIDGE_HEADER {
            error!("Catridge has an invalid header");
            return Err(IronNesError::CartridgeError);
        }
        Ok(())
    }

    pub fn get_prog_size(&self) -> usize {
        Self::CHIP_SIZE_PROG * self.num_prog_rom
    }

    pub fn get_ppu_size(&self) -> usize {
        Self::CHIP_SIZE_PPU * self.num_ppu_vrom
    }

    pub fn get_ram_size(&self) -> usize {
        Self::CHIP_SIZE_RAM * self.num_ram
    }
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cartridge {:04x} kB ROM {:04x} kB VROM {:04x} kB RAM",
            self.get_prog_size(),
            self.get_ppu_size(),
            self.get_ram_size(),
        )?;

        match self.mirror {
            MirrorDirection::Horizontal => write!(f, " MIRROR_HORIZONTAL")?,
            MirrorDirection::Vertical => write!(f, " MIRROR_VERTICAL")?,
            MirrorDirection::FourScreen => write!(f, " FOUR_SCREEN")?,
        }

        if self.has_battery {
            write!(f, " BATTERY")?;
        }

        if self.has_trainer {
            write!(f, " TRAINER")?;
        }

        if self.has_trainer {
            write!(f, " TRAINER")?;
        }

        let result = match self.region {
            CartridgeRegion::PAL => write!(f, " PAL"),
            CartridgeRegion::NTSC => write!(f, " NTSC"),
        };

        write!(f, " MAPPER: {}", which_mapper(self.mapper))?;

        result
    }
}

#[derive(Clone)]
pub enum MirrorDirection {
    Vertical,
    Horizontal,
    FourScreen,
}

impl Default for MirrorDirection {
    fn default() -> Self {
        Self::Horizontal
    }
}

#[derive(Clone)]
pub enum CartridgeRegion {
    PAL,
    NTSC,
}

impl Default for CartridgeRegion {
    fn default() -> Self {
        Self::PAL
    }
}

fn which_mapper(mapper: u8) -> &'static str {
    match mapper {
        0 => "No mapper",
        1 => "Nintendo MMC1",
        2 => "CNROM switch",
        3 => "UNROM switch",
        4 => "Nintendo MMC3",
        5 => "Nintendo MMC5",
        6 => "FFE F4xxx",
        7 => "AOROM switch",
        8 => "FFE F3xxx",
        9 => "Nintendo MMC2",
        10 => "Nintendo MMC4",
        11 => "ColorDreams chip",
        12 => "FFE F6xxx",
        13 => "CPROM switch",
        15 => "100-in-1 switch",
        16 => "Bandai chip",
        17 => "FFE F8xxx",
        18 => "Jaleco SS8806 chip",
        19 => "Namcot 106 chip",
        20 => "Nintendo DiskSystem",
        21 => "Konami VRC4a",
        22 => "Konami VRC2a",
        23 => "Konami VRC2a",
        24 => "Konami VRC6",
        25 => "Konami VRC4b",
        32 => "Irem G-101 chip",
        33 => "Taito TC0190/TC0350",
        34 => "Nina-1 board",
        64 => "Tengen RAMBO-1 chip",
        65 => "Irem H-3001 chip",
        66 => "GNROM switch",
        67 => "SunSoft3 chip",
        68 => "SunSoft4 chip",
        69 => "SunSoft5 FME-7 chip",
        71 => "Camerica chip",
        78 => "Irem 74HC161/32-based",
        79 => "AVE Nina-3 board",
        81 => "AVE Nina-6 board",
        91 => "Pirate HK-SF3 chip",
        _ => "UNKNOWN",
    }
}
