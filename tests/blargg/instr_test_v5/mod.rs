use regex::Regex;
use std::fs::File;
use std::io::{prelude::*, BufReader};

use iron_nes::nes::cpu;

pub fn get_golden<'a>(golden_file: &'a str) -> impl Iterator<Item = (usize, cpu::Registers)> {
    let re = Regex::new(r"([0-9A-F]{4})  ([0-9A-Z]{2})  A:([0-9A-F]{2}).*X:([0-9A-F]{2}).*Y:([0-9A-F]{2}).*P:([0-9A-F]{2}).*SP:([0-9A-F]{2}).*CYC:.*([0-9]+)").unwrap();

    let file = File::open(golden_file).unwrap();
    let reader = BufReader::new(file);

    reader.lines().map(move |line| {
        let mut reg = cpu::Registers::new();

        let l = line.unwrap();
        let caps = re.captures(&l).unwrap();

        reg.pc = u16::from_str_radix(caps.get(1).unwrap().as_str(), 16).unwrap();
        let _instr = caps.get(2).unwrap().as_str(); // PROBABLY USELESS...
        reg.a = u8::from_str_radix(caps.get(3).unwrap().as_str(), 16).unwrap();
        reg.x = u8::from_str_radix(caps.get(4).unwrap().as_str(), 16).unwrap();
        reg.y = u8::from_str_radix(caps.get(5).unwrap().as_str(), 16).unwrap();
        let flags = u8::from_str_radix(caps.get(6).unwrap().as_str(), 16).unwrap();
        reg.set_status(flags);
        reg.sp = u16::from_str_radix(caps.get(7).unwrap().as_str(), 16).unwrap();
        let cyc: usize = caps.get(8).unwrap().as_str().parse().unwrap();

        (cyc, reg)
    })
}
