use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

use nix::sys::ptrace::{self, Options};
use nix::sys::signal::Signal;
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::Pid;

use crate::regs;

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

    ptrace::setoptions(pid, Options::PTRACE_O_TRACESYSGOOD | Options::PTRACE_O_TRACEEXEC)?;

    let mut in_syscall = false;
    let mut pending: Option<Signal> = None;

    loop {
        ptrace::syscall(pid, pending.take())?;

        match waitpid(pid, None)? {
            WaitStatus::PtraceSyscall(_) => {
                in_syscall = !in_syscall;

                let syscall = regs::read(pid)?;

                if in_syscall {
                    println!("enter syscall {}", syscall.nr);
                } else {
                    println!("exit syscall {} = {}", syscall.nr, syscall.ret);
                }
            }
            WaitStatus::Stopped(_, sig) => {
                pending = Some(sig);
            },
            WaitStatus::Exited(_, code) => return Ok(code),
            WaitStatus::Signaled(_, signal, _) => return Ok(128 + signal as i32),
            _ => {}
        }
    }
}
