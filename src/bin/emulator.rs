use clap;
use simplelog::*;
use std::fs::File;

mod debugger;

use iron_nes::error::*;
use iron_nes::nes::IronNes;

fn main() -> IronNesResult<()> {
    let yaml = clap::load_yaml!("emulator.yml");
    let matches = clap::App::from_yaml(yaml)
        .version(&*format!("v{}", clap::crate_version!()))
        .get_matches();

    let rom = matches.value_of("rom").unwrap();
    let log_level = match matches.occurrences_of("v") {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Trace,
    };

    let is_debug = matches.occurrences_of("debug") > 0;

    let term_logger = TermLogger::new(log_level, Config::default(), TerminalMode::Mixed).unwrap();
    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![term_logger];

    if let Some(logfile) = matches.value_of("log") {
        let file_logger = WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(logfile).unwrap(),
        );
        loggers.push(file_logger);
    }

    CombinedLogger::init(loggers).unwrap();

    let mut nes = IronNes::new();
    nes.boot(rom)?;

    match is_debug {
        true => {
            let mut debugger = debugger::IronNesDebugger::new();
            debugger::run_debugger(&mut nes, &mut debugger);
            Ok(())
        }
        _ => nes.run(),
    }
}
