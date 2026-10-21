mod tracer;

use std::env;
use std::process;

use nix::sys::ptrace;
use nix::sys::ptrace::Options;
use nix::sys::wait::{waitpid, WaitStatus};
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
        Ok(WaitStatus::Stopped(..)) => {}
        Ok(status) => {
            eprintln!("scully: expected initial stop, got: {status:?}");
            process::exit(1);
        }
        Err(error) => {
            eprintln!("scully: waitpid failed: {error}");
            process::exit(1);
        }
    }

    if let Err(error) = ptrace::setoptions(pid, Options::PTRACE_O_TRACESYSGOOD) {
        eprintln!("scully: setoptions failed: {error}");
        process::exit(1);
    }

    loop {
        if let Err(error) = ptrace::syscall(pid, None) {
            eprintln!("scully: resume failed: {error}");
            process::exit(1);
        }

        let status = match waitpid(pid, None) {
            Ok(status) => status,
            Err(error) => {
                eprintln!("scully: waitpid failed: {error}");
                process::exit(1);
            }
        };

        match status {
            WaitStatus::PtraceSyscall(_) => {
                match ptrace::getregs(pid) {
                    Ok(regs) => println!("syscall {}", regs.orig_rax),
                    Err(error) => {
                        eprintln!("scully: getregs failed: {error}");
                        process::exit(1);
                    }
                }
            }
            WaitStatus::Exited(..) | WaitStatus::Signaled(..) => break,
            _ => println!("wait status: {status:?}"),
        }
    }
}
