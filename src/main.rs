mod format;
mod mem;
mod regs;
mod stats_table;
mod syscall_info;
mod tracer;

use std::env;
use std::process;

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();

    let stats_only = args.first().is_some_and(|arg| arg == "-c");
    if stats_only {
        args.remove(0);
    }

    let Some((cmd, cmd_args)) = args.split_first() else {
        eprintln!("usage: scully [-c] <command> [args...]");
        process::exit(2);
    };

    let child = match tracer::spawn_traced(cmd, cmd_args) {
        Ok(child) => child,
        Err(error) => {
            eprintln!("scully: failed to execute '{cmd}': {error}");
            process::exit(1);
        }
    };

    match tracer::run(&child, stats_only) {
        Ok(code) => process::exit(code),
        Err(error) => {
            eprintln!("scully: {error}");
            process::exit(1);
        }
    }
}
