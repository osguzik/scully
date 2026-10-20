mod tracer;

use std::env;
use std::process;

use nix::sys::wait::waitpid;
use nix::unistd::Pid;

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

    let pid = Pid::from_raw(child.id() as i32);
    match waitpid(pid, None) {
        Ok(status) => println!("wait status: {status:?}"),
        Err(error) => {
            eprintln!("scully: waitpid failed: {error}");
            process::exit(1);
        }
    }
}
