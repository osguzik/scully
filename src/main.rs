use std::env;
use std::process::{self, Command};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let Some((cmd, cmd_args)) = args.split_first() else {
        eprintln!("usage: scully <command> [args...]");
        process::exit(2);
    };

    let status = match Command::new(cmd).args(cmd_args).status() {
        Ok(status) => status,
        Err(error) => {
            eprintln!("scully: failed to execute '{cmd}': {error}");
            process::exit(1);
        }
    };

    process::exit(status.code().unwrap_or(1));
}
