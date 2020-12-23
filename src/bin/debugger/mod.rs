use iron_nes::error::*;
use iron_nes::nes::IronNes;

use shrust::{Shell, ShellIO};
use std::io::prelude::*;

pub fn run_debugger<'a>(nes: &'a mut IronNes, debugger: &'a mut IronNesDebugger) {
    let mut shell = Shell::new((debugger, nes));

    shell.new_command("b", "add breakpoint", 1, |io, (d, _), s| {
        let addr = usize::from_str_radix(s[0], 16).unwrap();
        d.add_breakpoint(addr);
        writeln!(io, "breakpoint set {:04x}", addr)?;
        Ok(())
    });

    shell.new_command("wc", "add watch cycle", 1, |io, (d, _), s| {
        let cycle = usize::from_str_radix(s[0], 10).unwrap();
        d.add_watch_cycle(cycle);
        writeln!(io, "watch cycle set {}", cycle)?;
        Ok(())
    });

    shell.new_command_noargs("r", "run", |io, (d, nes)| {
        loop {
            match d.step(nes).unwrap() {
                DebuggerState::Breakpoint(addr) => {
                    writeln!(io, "breakpoint hit {:04x}", addr)?;
                    break;
                }
                DebuggerState::WatchCycle(cycle) => {
                    writeln!(io, "watch cycle hit {}", cycle)?;
                    break;
                }
                DebuggerState::Stopped => continue,
            }
        }
        Ok(())
    });

    shell.new_command_noargs("s", "step", |io, (d, nes)| {
        match d.step(nes).unwrap() {
            DebuggerState::Breakpoint(addr) => {
                writeln!(io, "breakpoint hit {:04x}", addr)?;
            }
            DebuggerState::WatchCycle(cycle) => {
                writeln!(io, "watch cycle hit {}", cycle)?;
            }
            _ => (),
        }
        Ok(())
    });

    shell.new_command("p", "print addr -> range", 2, |io, (_, nes), s| {
        let addr = usize::from_str_radix(s[0], 16).unwrap();
        let range = usize::from_str_radix(s[1], 10).unwrap();
        const TERM_WIDTH: usize = 8;

        for i in 0..range {
            if i % TERM_WIDTH == 0 {
                write!(io, "{:04x}   ", addr + i)?;
            }

            write!(io, "{:02x} ", nes.peek(addr + i).unwrap())?;

            if i % TERM_WIDTH == (TERM_WIDTH - 1) {
                writeln!(io, "")?;
            }
        }
        if range % TERM_WIDTH != 0 {
            writeln!(io, "")?;
        }
        Ok(())
    });

    shell.run_loop(&mut ShellIO::default());
}

enum DebuggerState {
    Stopped,
    Breakpoint(usize),
    WatchCycle(usize),
}

pub struct IronNesDebugger {
    breakpoints: Vec<usize>,
    watch_cycles: Vec<usize>,
}

impl IronNesDebugger {
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            watch_cycles: Vec::new(),
        }
    }

    /// Returns if a breakpoint was hit, and what PC was when it happened
    fn step<'a>(&mut self, nes: &'a mut IronNes) -> IronNesResult<DebuggerState> {
        nes.step()?;
        let pc = nes.get_cpu_registers().pc as usize;
        if self.is_breakpoint_hit(pc) {
            return Ok(DebuggerState::Breakpoint(pc));
        }

        let cycle = nes.get_cycles();
        if self.is_watch_cycle_hit(cycle) {
            return Ok(DebuggerState::WatchCycle(cycle));
        }

        Ok(DebuggerState::Stopped)
    }

    fn is_breakpoint_hit(&self, addr: usize) -> bool {
        self.breakpoints.iter().any(|x| *x == addr)
    }

    fn is_watch_cycle_hit(&self, cycle: usize) -> bool {
        self.watch_cycles.iter().any(|x| *x == cycle)
    }

    fn add_breakpoint(&mut self, addr: usize) {
        self.breakpoints.push(addr);
    }

    fn add_watch_cycle(&mut self, cycle: usize) {
        self.watch_cycles.push(cycle);
    }
}
