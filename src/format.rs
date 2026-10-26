use nix::errno::Errno;
use nix::libc;
use nix::unistd::Pid;

use crate::mem;
use crate::regs::RawSyscall;
use crate::syscall_info::{self, ArgKind, FlagNames, OPEN_FLAGS};

/// Linux reserves return values from -1 to -MAX_ERRNO for errors
const MAX_ERRNO: i64 = 4095;

/// Buffers longer than MAX_BUF_SHOWN are cut off and end with "..."
const MAX_BUF_SHOWN: usize = 32;

pub fn call(pid: Pid, call: &RawSyscall, ret: Option<i64>) -> String {
    format!("{}({})", syscall_info::name(call.nr), args(pid, call, ret))
}

pub fn ret(ret: i64) -> String {
    if (-MAX_ERRNO..0).contains(&ret) {
        let errno = Errno::from_raw(-ret as i32);
        format!("-1 {errno:?} ({})", errno.desc())
    } else {
        ret.to_string()
    }
}

fn args(pid: Pid, syscall: &RawSyscall, ret: Option<i64>) -> String {
    let args: Vec<String> = match syscall_info::arg_kinds(syscall.nr) {
        Some(kinds) => kinds
            .iter()
            .zip(syscall.args)
            .map(|(kind, val)| arg(pid, *kind, val, syscall, ret))
            .collect(),
        None => syscall.args.iter().map(|val| format!("{val:#x}")).collect(),
    };

    args.join(", ")
}

fn arg(pid: Pid, kind: ArgKind, val: u64, syscall: &RawSyscall, ret: Option<i64>) -> String {
    match kind {
        ArgKind::Hex => {
            if val == 0 {
                "0".to_string()
            } else {
                format!("{val:#x}")
            }
        }
        ArgKind::Int => (val as i32).to_string(),
        ArgKind::UInt => val.to_string(),
        ArgKind::Str => match mem::read_string(pid, val) {
            Some(bytes) => quote(&bytes),
            None => format!("{val:#x}"),
        }
        ArgKind::Ptr => {
            if val == 0 {
                "NULL".to_string()
            } else {
                format!("{val:#x}")
            }
        }
        ArgKind::Buf(count_idx) => buf(pid, val, syscall.args[count_idx]),
        ArgKind::BufRet => match ret {
            Some(count) if count >= 0 => buf(pid, val, count as u64),
            _ => format!("{val:#x}"),
        }
        ArgKind::Flags(known_flags) => flags(val, known_flags),
        ArgKind::OpenFlags => open_flags(val),
    }
}

fn buf(pid: Pid, addr: u64, count: u64) -> String {
    let count = count as usize;
    let shown_count = count.min(MAX_BUF_SHOWN);

    match mem::read_bytes(pid, addr, shown_count) {
        Some(bytes) => {
            let text = quote(&bytes);
            if shown_count < count {
                format!("{text}...")
            } else {
                text
            }
        }
        None => format!("{addr:#x}"),
    }
}

fn flags(val: u64, known_flags: FlagNames) -> String {
    let val = val as u32;
    let mut parts = Vec::new();
    let mut unknown = val;

    for &(bit, name) in known_flags {
        if (val & bit) == bit {
            parts.push(name.to_string());
            unknown &= !bit;
        }
    }

    if unknown != 0 || parts.is_empty() {
        parts.push(format!("{unknown:#x}"));
    }

    parts.join("|")
}

fn open_flags(val: u64) -> String {
    let val = val as u32;
    let acc_mode = val & libc::O_ACCMODE as u32;
    let other_flags = val & !(libc::O_ACCMODE as u32);

    let mode = match acc_mode as i32 {
        libc::O_RDONLY => "O_RDONLY",
        libc::O_WRONLY => "O_WRONLY",
        libc::O_RDWR => "O_RDWR",
        _ => "O_ACCMODE",
    };

    if other_flags == 0 {
        mode.to_string()
    } else {
        format!("{mode}|{}", flags(other_flags as u64, OPEN_FLAGS))
    }
}

fn quote(bytes: &[u8]) -> String {
    format!("\"{}\"", bytes.escape_ascii())
}
