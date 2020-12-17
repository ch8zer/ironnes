use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

use csv::ReaderBuilder;

fn main() {
    let instr_legal = include_str!("src/nes/cpu/6502ops.csv");
    let mut rdr = ReaderBuilder::new().from_reader(instr_legal.as_bytes());
    let legal = rdr.records().map(|it| csv_to_instr(&it.unwrap(), true));

    let instr_illegal = include_str!("src/nes/cpu/unofficial.csv");
    let mut rdr = ReaderBuilder::new().from_reader(instr_illegal.as_bytes());
    let illegal = rdr.records().map(|it| csv_to_instr(&it.unwrap(), false));

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let op_dest_path = Path::new(&out_dir).join("instruction_lookup.rs");
    let cpu_dest_path = Path::new(&out_dir).join("instr_jumptable.rs");
    let mut op_file = fs::File::create(&op_dest_path).unwrap();
    let mut cpu_file = fs::File::create(&cpu_dest_path).unwrap();

    op_file
        .write(
            b"
fn lookup_instr(opcode: u8) -> Instruction {
    match opcode {
",
        )
        .unwrap();

    cpu_file
        .write(
            b"
    match opcode {
",
        )
        .unwrap();

    legal.chain(illegal).for_each(|(opcode, name, instr)| {
        // OP FILE
        let line = format!("{} => {},\n", opcode, instr);
        op_file.write(line.as_bytes()).unwrap();

        // CPU SWITCH
        let line = format!(
            "0x{:02x} => {}_execute(self, &instr, mem),\n",
            opcode,
            name.to_lowercase()
        );
        cpu_file.write(line.as_bytes()).unwrap();
    });

    op_file
        .write(
            b"
        _ => Instruction::illegal(opcode),
    }
}
",
        )
        .unwrap();

    cpu_file
        .write(
            b"
    _ => Err(IronNesError::IllegalInstruction),
}
",
        )
        .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}

fn csv_to_instr(record: &csv::StringRecord, is_legal: bool) -> (u8, String, String) {
    let opcode = record[0].trim_start_matches("0x");
    let opcode = u8::from_str_radix(opcode, 16).unwrap();

    let mut mnemonic = record[1].to_string();
    let opname = mnemonic.clone();
    if !is_legal {
        mnemonic = format!("*{}", mnemonic);
    }

    let addr_mode = addr_mode_str(&record[2]);
    let bytes = record[3].parse::<u8>().unwrap();

    let cycles_str = &record[4];
    let (cycles, can_cross) = match cycles_str.find('/') {
        Some(idx) => (&cycles_str[0..idx], true),
        _ => (cycles_str, false),
    };
    let cycles = cycles.parse::<usize>().unwrap();

    let instr = format!(
        "Instruction::new({}, \"{}\", {}, {}, {}, {})",
        opcode, mnemonic, bytes, cycles, can_cross, addr_mode,
    );

    (opcode, opname, instr)
}

fn addr_mode_str(input: &str) -> &str {
    match input {
        "IMP" => "AddressingMode::Implied",
        "ACC" => "AddressingMode::Accumulator",
        "IMM" => "AddressingMode::Immediate",
        "ABS" => "AddressingMode::Absolute",
        "ABSX" => "AddressingMode::AbsoluteX",
        "ABSY" => "AddressingMode::AbsoluteY",
        "IND" => "AddressingMode::Indirect",
        "INDX" => "AddressingMode::IndirectX",
        "INDY" => "AddressingMode::IndirectY",
        "ZP" => "AddressingMode::ZeroPage",
        "ZPX" => "AddressingMode::ZeroPageX",
        "ZPY" => "AddressingMode::ZeroPageY",
        "REL" => "AddressingMode::Relative",
        "ILL" => "AddressingMode::Illegal",
        _ => "AddressingMode::Unknown",
    }
}
