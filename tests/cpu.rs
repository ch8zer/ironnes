use log::*;
use simplelog::*;
use std::path::PathBuf;
use std::sync::Once;

use iron_nes::error::*;
use iron_nes::nes::cpu;
use iron_nes::nes::memory;
use iron_nes::nes::IronNes;

mod blargg;
mod nestest;

static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| {
        CombinedLogger::init(vec![TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
        )
        .unwrap()])
        .unwrap();
    });
}

fn get_filename(parts: &[&str]) -> String {
    parts
        .iter()
        .collect::<PathBuf>()
        .into_os_string()
        .into_string()
        .unwrap()
}

// Test runner
// load: where to preload the PC
fn run_test(
    rom: String,
    golden: impl Iterator<Item = (usize, cpu::Registers)>,
    load: Option<memory::Addr>,
    can_count_cycles: bool,
) -> IronNesResult<IronNes> {
    setup();

    let mut nes = IronNes::new(&rom);
    nes.reset()?;

    if let Some(x) = load {
        nes.jsr(x)?;
    }

    golden.for_each(|(golden_cyc, golden_reg)| {
        let regs = nes.get_cpu_registers();
        let reg_p = regs.get_status();
        let cpu_cycles = nes.get_cycles();

        assert_eq!(
            golden_reg.pc, regs.pc,
            "PC mismatch expected: {:04x} actual: {:04x}",
            golden_reg.pc, regs.pc
        );
        assert_eq!(
            golden_reg.a, regs.a,
            "A mismatch expected: {:02x} actual: {:02x}",
            golden_reg.a, regs.a
        );
        assert_eq!(
            golden_reg.x, regs.x,
            "X mismatch expected: {:02x} actual: {:02x}",
            golden_reg.x, regs.x
        );
        assert_eq!(
            golden_reg.y, regs.y,
            "Y mismatch expected: {:02x} actual: {:02x}",
            golden_reg.y, regs.y
        );
        assert_eq!(
            golden_reg.sp, regs.sp,
            "SP mismatch expected: {:02x} actual: {:02x}",
            golden_reg.sp, regs.sp
        );
        assert_eq!(
            golden_reg.get_status(),
            reg_p,
            "P mismatch expected: {:08b} actual: {:08b}",
            golden_reg.get_status(),
            reg_p
        );
        if can_count_cycles {
            assert_eq!(
                golden_cyc, cpu_cycles,
                "CPU CYCLE mismatch expected: {} actual: {}",
                golden_cyc, cpu_cycles
            );
        }

        nes.step().unwrap();
    });

    Ok(nes)
}

/*
 * NESTEST
 * Test Instructions & Result Codes: https://www.qmtpro.com/~nes/misc/nestest.txt
 */
#[test]
fn cpu_nestest() -> IronNesResult<()> {
    setup();

    let rom = get_filename(&[env!("CARGO_MANIFEST_DIR"), "tests/nestest/nestest.nes"]);
    let golden_path = get_filename(&[env!("CARGO_MANIFEST_DIR"), "tests/nestest/nestest.log"]);

    let _nes = run_test(rom, nestest::get_golden(&golden_path), Some(0xc000), true)?;
    Ok(())
}

/*
 * BLARGG CPU Tests
 * http://forums.nesdev.com/viewtopic.php?t=7048
 */
fn run_blargg_test(rom_file: String, golden_file: String) -> IronNesResult<()> {
    let mut nes = run_test(
        rom_file,
        blargg::instr_test_v5::get_golden(&golden_file),
        None,
        false,
    )?;
    assert_eq!(0, nes.peek(0x6000)?);
    Ok(())
}

//#[test]
//fn cpu_blargg_all_instr() -> IronNesResult<()> {
//    run_blargg_test(
//        "instr_test_v5/all_instrs.nes"
//    )
//}
//
//#[test]
//fn cpu_blargg_official_instr() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/official_only.nes",
//    ])
//}

// TODO
//#[test]
//fn cpu_blargg_01_basics() -> IronNesResult<()> {
//    run_blargg_test(
//        get_filename(&[
//            env!("CARGO_MANIFEST_DIR"),
//            "tests/blargg/instr_test_v5/rom_singles/01-basics.nes",
//        ]),
//        get_filename(&[
//            env!("CARGO_MANIFEST_DIR"),
//            "tests/blargg/instr_test_v5/goldens/01-basics.log",
//        ]),
//    )
//}

//#[test]
//fn cpu_blargg_02_implied() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/02-implied.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_03_immediate() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/03-immediate.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_04_zero_page() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/04-zero_page.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_05_zp_xy() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/05-zp_xy.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_06_absolute() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/06-absolute.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_07_abs_xy() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/07-abs_xy.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_08_ind_x() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/08-ind_x.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_09_ind_y() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/09-ind_y.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_10_branches() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/10-branches.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_11_stack() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/11-stack.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_12_jmp_jsr() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/12-jmp_jsr.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_13_rts() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/13-rts.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_14_rti() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/14-rti.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_15_brk() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/15-brk.nes",
//    ])
//}
//
//#[test]
//fn cpu_blargg_16_special() -> IronNesResult<()> {
//    run_blargg_test(&[
//        env!("CARGO_MANIFEST_DIR"),
//        "tests/blargg/instr_test_v5/rom_singles/16-special.nes",
//    ])
//}
