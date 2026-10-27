use nix::errno::Errno;
use nix::libc;
use nix::unistd::Pid;

use crate::mem;
use crate::regs::RawSyscall;
use crate::syscall_info::{self, ArgKind, FlagNames, OPEN_FLAGS};

/// Buffers longer than MAX_BUF_SHOWN are cut off and end with "..."
const MAX_BUF_SHOWN: usize = 32;

pub fn call(pid: Pid, call: &RawSyscall, ret: Option<i64>) -> String {
    format!("{}({})", syscall_info::name(call.nr), args(pid, call, ret))
}

pub fn ret(nr: u64, val: i64) -> String {
    if syscall_info::is_error(val) {
        let errno = Errno::from_raw(-val as i32);
        format!("-1 {errno:?} ({})", errno.desc())
    } else if syscall_info::returns_ptr(nr) {
        format!("{val:#x}")
    } else {
        val.to_string()
    }
}

fn args(pid: Pid, syscall: &RawSyscall, ret: Option<i64>) -> String {
    let args: Vec<String> = match syscall_info::arg_kinds(syscall.nr) {
        Some(kinds) => {
            let kinds = if mode_unused(syscall) {
                &kinds[..3]
            } else {
                kinds
            };

            kinds
                .iter()
                .zip(syscall.args)
                .map(|(kind, val)| arg(pid, *kind, val, syscall, ret))
                .collect()
        }
        None => syscall.args.iter().map(|val| hex(*val)).collect(),
    };

    args.join(", ")
}

fn mode_unused(syscall: &RawSyscall) -> bool {
    if syscall.nr != libc::SYS_openat as u64 {
        return false;
    }

    let flags = syscall.args[2] as i32;
    let can_create = (flags & libc::O_CREAT) != 0 || (flags & libc::O_TMPFILE) == libc::O_TMPFILE;

    !can_create
}

fn arg(pid: Pid, kind: ArgKind, val: u64, syscall: &RawSyscall, ret: Option<i64>) -> String {
    match kind {
        ArgKind::Hex => hex(val),
        ArgKind::Int => (val as i32).to_string(),
        ArgKind::UInt => val.to_string(),
        ArgKind::Str => match mem::read_string(pid, val) {
            Some(bytes) => quote(&bytes),
            None => format!("{val:#x}"),
        },
        ArgKind::Ptr => {
            if val == 0 {
                "NULL".to_string()
            } else {
                format!("{val:#x}")
            }
        }
        ArgKind::DirFd => {
            let fd = val as i32;
            if fd == libc::AT_FDCWD {
                "AT_FDCWD".to_string()
            } else {
                fd.to_string()
            }
        }
        ArgKind::Buf(count_idx) => buf(pid, val, syscall.args[count_idx]),
        ArgKind::BufRet => match ret {
            Some(count) if count >= 0 => buf(pid, val, count as u64),
            _ => format!("{val:#x}"),
        },
        ArgKind::Flags(known_flags) => flags(val as u32, known_flags),
        ArgKind::OpenFlags => open_flags(val as u32),
        ArgKind::Mode => format!("0{:o}", val as u32),
    }
}

fn hex(val: u64) -> String {
    if val == 0 {
        "0".to_string()
    } else {
        format!("{val:#x}")
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

fn flags(val: u32, known_flags: FlagNames) -> String {
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

fn open_flags(val: u32) -> String {
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
        format!("{mode}|{}", flags(other_flags, OPEN_FLAGS))
    }
}

fn quote(bytes: &[u8]) -> String {
    format!("\"{}\"", bytes.escape_ascii())
}
