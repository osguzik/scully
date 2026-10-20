mod tracer;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let Some((cmd, cmd_args)) = args.split_first() else {
        eprintln!("usage: scully <command> [args...]");
        process::exit(2);
    };

    let child = match tracer::spawn_traced(cmd, cmd_args) {
        Ok(child) => child,
        Err(error) => {
            eprintln!("scully: failed to execute '{cmd}': {error}");
            process::exit(1);
        }
    };

    println!("spawned traced child with pid {}", child.id());
}
