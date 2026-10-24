use nix::errno::Errno;
use nix::unistd::Pid;

use crate::mem;
use crate::regs::RawSyscall;
use crate::syscall_info::{self, ArgKind};

/// Linux reserves return values from -1 to -MAX_ERRNO for errors
const MAX_ERRNO: i64 = 4095;

/// Buffers longer than MAX_BUF_SHOWN are cut off and end with "..."
const MAX_BUF_SHOWN: usize = 32;

pub fn call(pid: Pid, call: &RawSyscall) -> String {
    format!("{}({})", syscall_info::name(call.nr), args(pid, call))
}

pub fn ret(ret: i64) -> String {
    if (-MAX_ERRNO..0).contains(&ret) {
        let errno = Errno::from_raw(-ret as i32);
        format!("-1 {errno:?} ({})", errno.desc())
    } else {
        ret.to_string()
    }
}

fn args(pid: Pid, syscall: &RawSyscall) -> String {
    let args: Vec<String> = match syscall_info::arg_kinds(syscall.nr) {
        Some(kinds) => kinds
            .iter()
            .zip(syscall.args)
            .map(|(kind, val)| arg(pid, *kind, val, syscall))
            .collect(),
        None => syscall.args.iter().map(|val| format!("{val:#x}")).collect(),
    };

    args.join(", ")
}

fn arg(pid: Pid, kind: ArgKind, val: u64, syscall: &RawSyscall) -> String {
    match kind {
        ArgKind::Hex => format!("{val:#x}"),
        ArgKind::Int => (val as i32).to_string(),
        ArgKind::UInt => val.to_string(),
        ArgKind::Str => match mem::read_string(pid, val) {
            Some(bytes) => format!("{:?}", String::from_utf8_lossy(&bytes)),
            None => format!("{val:#x}"),
        },
        ArgKind::Buf(count_idx) => buf(pid, val, syscall.args[count_idx]),
    }
}

fn buf(pid: Pid, addr: u64, count: u64) -> String {
    let count = count as usize;
    let shown_count = count.min(MAX_BUF_SHOWN);

    match mem::read_bytes(pid, addr, shown_count) {
        Some(bytes) => {
            let text = format!("{:?}", String::from_utf8_lossy(&bytes));
            if shown_count < count {
                format!("{text}...")
            } else {
                text
            }
        }
        None => format!("{addr:#x}"),
    }
}
