use clap;
use iron_nes::error::*;
use log::*;
use simplelog::*;

use iron_nes::nes::cartridge::Cartridge;
use iron_nes::nes::cpu::instruction::Instruction;
use iron_nes::nes::cpu::Cpu;
use iron_nes::nes::memory::{Addr, Memory};

fn main() -> IronNesResult<()> {
    let yaml = clap::load_yaml!("disassembler.yml");
    let matches = clap::App::from_yaml(yaml)
        .version(&*format!("v{}", clap::crate_version!()))
        .get_matches();

    let cartridge_file = matches.value_of("rom").unwrap();
    let log_level = match matches.occurrences_of("v") {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Trace,
    };

    // Init the logger
    CombinedLogger::init(vec![TermLogger::new(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
    )
    .unwrap()])
    .unwrap();

    let (cartridge, prog_rom, _) =
        Cartridge::load(cartridge_file).expect("Failed to load cartridge");
    let mut mem = Memory::new();
    mem.load_rom(&prog_rom)?;

    println!("NMI {:04x}", mem.load16(Cpu::ADDR_NMI)?);
    println!("RESET {:04x}", mem.load16(Cpu::ADDR_RESET)?);
    println!("IRQ {:04x}", mem.load16(Cpu::ADDR_IRQ)?);

    let (mut pc, end) = (0xc000, 0xFFF0);
    while pc < end {
        let opcode = mem.load(pc)?;
        let instr = Instruction::lookup(opcode);
        println!("{:04x} {}", pc, instr.print(pc, &mem));
        pc += instr.bytes as Addr;
    }

    // Bigger than one page
    if cartridge.get_prog_size() > Cartridge::CHIP_SIZE_PROG {
        let (mut pc, end) = (0x8000, 0xBFFF);
        while pc < end {
            let opcode = mem.load(pc)?;
            let instr = Instruction::lookup(opcode);
            println!("{:04x} {}", pc, instr.print(pc, &mem));
            pc += instr.bytes as Addr;
        }
    }

    Ok(())
}
