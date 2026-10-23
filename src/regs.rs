use std::io;

use nix::sys::ptrace;
use nix::unistd::Pid;

pub struct RawSyscall {
    pub nr: u64,
    pub args: [u64; 6],
    pub ret: i64,
}

pub fn read(pid: Pid) -> io::Result<RawSyscall> {
    let regs = ptrace::getregs(pid)?;

    Ok(RawSyscall {
        nr: regs.orig_rax,
        args: [regs.rdi, regs.rsi, regs.rdx, regs.r10, regs.r8, regs.r9],
        ret: regs.rax as i64,
    })
}
