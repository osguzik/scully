use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

use nix::errno::Errno;
use nix::sys::ptrace::{self, Options};
use nix::sys::signal::Signal;
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::Pid;

use crate::regs::{self, RawSyscall};
use crate::syscall_info;

pub fn spawn_traced(cmd: &str, cmd_args: &[String]) -> io::Result<Child> {
    let mut command = Command::new(cmd);
    command.args(cmd_args);

    // SAFETY: runs between fork and exec and only calls ptrace(PTRACE_TRACEME)
    unsafe {
        command.pre_exec(|| ptrace::traceme().map_err(io::Error::from));
    }

    command.spawn()
}

pub fn run(child: &Child) -> io::Result<i32> {
    let pid = Pid::from_raw(child.id() as i32);

    match waitpid(pid, None)? {
        WaitStatus::Stopped(..) => {}
        status => {
            return Err(io::Error::other(format!(
                "expected initial stop, got: {status:?}"
            )));
        }
    }

    ptrace::setoptions(
        pid,
        Options::PTRACE_O_TRACESYSGOOD | Options::PTRACE_O_TRACEEXEC,
    )?;

    let mut entry: Option<RawSyscall> = None;
    let mut pending: Option<Signal> = None;

    loop {
        match ptrace::syscall(pid, pending.take()) {
            Ok(()) | Err(Errno::ESRCH) => {}
            Err(error) => return Err(error.into()),
        }

        match waitpid(pid, None)? {
            WaitStatus::PtraceSyscall(_) => {
                let syscall = regs::read(pid)?;

                match entry.take() {
                    None => entry = Some(syscall),
                    Some(start) => eprintln!("{} = {}", format_call(&start), syscall.ret),
                }
            }

            WaitStatus::Stopped(_, sig) => {
                eprintln!("--- {sig:?} ---");
                pending = Some(sig);
            }

            WaitStatus::Exited(_, code) => {
                if let Some(start) = entry {
                    eprintln!("{} = ?", format_call(&start));
                }
                eprintln!("+++ exited with {code} +++");
                return Ok(code);
            }

            WaitStatus::Signaled(_, sig, _) => {
                if let Some(start) = entry {
                    eprintln!("{} = ?", format_call(&start));
                }
                eprintln!("+++ killed by {sig:?} +++");
                return Ok(128 + sig as i32);
            }

            _ => {}
        }
    }
}

fn format_call(call: &RawSyscall) -> String {
    format!(
        "{}({})",
        syscall_info::name(call.nr),
        format_args(&call.args)
    )
}

fn format_args(args: &[u64; 6]) -> String {
    let args: Vec<String> = args.iter().map(|arg| format!("{arg:#x}")).collect();
    args.join(", ")
}
