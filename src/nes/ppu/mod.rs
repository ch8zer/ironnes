mod registers;

use crate::nes::bus::memory_mapped::*;
use crate::nes::cartridge::Cartridge;

//pub struct Frame(pub Vec<u8>);
//
//impl Frame {
//    const WIDTH: usize = 256 * 2;
//    const HEIGHT: usize = 240;
//
//    fn new() -> Self {
//        Frame {
//            0: vec![0; Self::WIDTH * Self::HEIGHT * 3],
//        }
//    }
//
//    fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
//        let offset = y * Self::WIDTH + x;
//        let offset = offset * 3;
//
//        self.0[offset] = rgb.0;
//        self.0[offset + 1] = rgb.1;
//        self.0[offset + 2] = rgb.2;
//    }
//}

pub struct Ppu;

impl Ppu {
    pub fn new() -> Self {
        Self {}
    }

    /**
     * # Returns
     * * allocated memory for devices (registers, nametables)
     */
    pub fn alloc_mem_devices(_cartridge: &Cartridge) -> (MemMappedDevice, MemMappedDevice) {
        // TODO this is based on mirroring, just allocate a
        // full and empty array for now
        let nametables = Box::new(MemoryMappedRam::new(0x400 * 4));
        let reg = Box::new(registers::Registers::new());
        (reg, nametables)
    }

    //pub fn render(&self) -> Frame {
    //    let mut frame = Frame::new();

    //    let num_rows = Frame::WIDTH / 8 / 2;
    //    let num_cols = Frame::HEIGHT / 8;

    //    for i in 0..256 {
    //        let frame_x = i % num_rows;
    //        let frame_y = i / num_rows;
    //        self.load_tile(&mut frame, 8 * frame_x, 8 * frame_y, 0, i);
    //    }

    //    frame
    //}

    //fn load_tile(
    //    &self,
    //    frame: &mut Frame,
    //    frame_x: usize,
    //    frame_y: usize,
    //    bank: usize,
    //    n_tile: usize,
    //) {
    //    let bank = bank * 1000usize;

    //    let tile = &self.vram[(bank + n_tile * 16)..(bank + n_tile * 16 + 16)];

    //    for y in 0..=7 {
    //        let mut upper = tile[y];
    //        let mut lower = tile[y + 8];

    //        for x in (0..=7).rev() {
    //            let value = ((1 & upper) << 1) | (1 & lower);
    //            upper = upper >> 1;
    //            lower = lower >> 1;

    //            let rgb = match value {
    //                0 => PALLETE[0x01],
    //                1 => PALLETE[0x23],
    //                2 => PALLETE[0x27],
    //                3 => PALLETE[0x30],
    //                _ => panic!("Impossible palette {}", value),
    //            };

    //            frame.set_pixel(frame_x + x, frame_y + y, rgb);
    //        }
    //    }
    //}
}

#[rustfmt::skip]
pub static PALLETE: [(u8,u8,u8); 64] = [
   (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
   (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
   (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
   (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
   (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
   (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
   (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
   (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
   (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
   (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
   (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
   (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
   (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];
