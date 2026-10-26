use nix::libc;
use syscalls::Sysno;

pub type FlagNames = &'static [(u32, &'static str)];

pub const PROT_FLAGS: FlagNames = &[
    (libc::PROT_READ as u32, "PROT_READ"),
    (libc::PROT_WRITE as u32, "PROT_WRITE"),
    (libc::PROT_EXEC as u32, "PROT_EXEC"),
];

pub const MAP_FLAGS: FlagNames = &[
    (libc::MAP_SHARED as u32, "MAP_SHARED"),
    (libc::MAP_PRIVATE as u32, "MAP_PRIVATE"),
    (libc::MAP_FIXED as u32, "MAP_FIXED"),
    (libc::MAP_ANONYMOUS as u32, "MAP_ANONYMOUS"),
    (libc::MAP_GROWSDOWN as u32, "MAP_GROWSDOWN"),
    (libc::MAP_DENYWRITE as u32, "MAP_DENYWRITE"),
    (libc::MAP_EXECUTABLE as u32, "MAP_EXECUTABLE"),
    (libc::MAP_LOCKED as u32, "MAP_LOCKED"),
    (libc::MAP_NORESERVE as u32, "MAP_NORESERVE"),
    (libc::MAP_POPULATE as u32, "MAP_POPULATE"),
    (libc::MAP_STACK as u32, "MAP_STACK"),
    (libc::MAP_FIXED_NOREPLACE as u32, "MAP_FIXED_NOREPLACE"),
];

pub const OPEN_FLAGS: FlagNames = &[
    (libc::O_CREAT as u32, "O_CREAT"),
    (libc::O_EXCL as u32, "O_EXCL"),
    (libc::O_NOCTTY as u32, "O_NOCTTY"),
    (libc::O_TRUNC as u32, "O_TRUNC"),
    (libc::O_APPEND as u32, "O_APPEND"),
    (libc::O_NONBLOCK as u32, "O_NONBLOCK"),
    (libc::O_DSYNC as u32, "O_DSYNC"),
    (libc::O_DIRECT as u32, "O_DIRECT"),
    (libc::O_DIRECTORY as u32, "O_DIRECTORY"),
    (libc::O_NOFOLLOW as u32, "O_NOFOLLOW"),
    (libc::O_NOATIME as u32, "O_NOATIME"),
    (libc::O_CLOEXEC as u32, "O_CLOEXEC"),
    (libc::O_PATH as u32, "O_PATH"),
];

#[derive(Clone, Copy)]
pub enum ArgKind {
    Hex,
    Int,
    UInt,
    Str,
    Ptr,
    DirFd,
    Buf(usize),
    BufRet,
    Flags(FlagNames),
    OpenFlags,
    Mode,
}

pub fn name(nr: u64) -> String {
    match Sysno::new(nr as usize) {
        Some(sysno) => sysno.name().to_string(),
        None => format!("syscall_{nr:#x}"),
    }
}

pub fn arg_kinds(nr: u64) -> Option<&'static [ArgKind]> {
    use ArgKind::*;

    let kinds: &'static [ArgKind] = match Sysno::new(nr as usize)? {
        // files
        Sysno::access => &[Str, Hex],
        Sysno::close => &[Int],
        Sysno::fcntl => &[Int, Int, Hex],
        Sysno::fstat => &[Int, Ptr],
        Sysno::getdents64 => &[Int, Ptr, UInt],
        Sysno::ioctl => &[Int, Hex, Hex],
        Sysno::newfstatat => &[DirFd, Str, Ptr, Hex],
        Sysno::openat => &[DirFd, Str, OpenFlags, Mode],
        Sysno::pread64 => &[Int, BufRet, UInt, UInt],
        Sysno::read => &[Int, BufRet, UInt],
        Sysno::write => &[Int, Buf(2), UInt],

        // memory
        Sysno::brk => &[Ptr],
        Sysno::mmap => &[Ptr, UInt, Flags(PROT_FLAGS), Flags(MAP_FLAGS), Int, Hex],
        Sysno::mprotect => &[Ptr, UInt, Flags(PROT_FLAGS)],
        Sysno::munmap => &[Ptr, UInt],

        // process
        Sysno::arch_prctl => &[Hex, Ptr],
        Sysno::exit_group => &[Int],
        Sysno::getrandom => &[Ptr, UInt, Hex],
        Sysno::prlimit64 => &[Int, Int, Ptr, Ptr],
        Sysno::rseq => &[Ptr, UInt, Hex, Hex],
        Sysno::set_robust_list => &[Ptr, UInt],
        Sysno::set_tid_address => &[Ptr],

        _ => return None,
    };

    Some(kinds)
}

pub fn returns_ptr(nr: u64) -> bool {
    matches!(Sysno::new(nr as usize), Some(Sysno::mmap | Sysno::brk))
}
