use syscalls::Sysno;

#[derive(Clone, Copy)]
pub enum ArgKind {
    Hex,
    Int,
    UInt,
    Str,
    Buf(usize),
    BufRet,
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
        Sysno::fstat => &[Int, Hex],
        Sysno::getdents64 => &[Int, Hex, UInt],
        Sysno::ioctl => &[Int, Hex, Hex],
        Sysno::newfstatat => &[Int, Str, Hex, Hex],
        Sysno::openat => &[Int, Str, Hex, Hex],
        Sysno::pread64 => &[Int, BufRet, UInt, UInt],
        Sysno::read => &[Int, BufRet, UInt],
        Sysno::write => &[Int, Buf(2), UInt],

        // memory
        Sysno::brk => &[Hex],
        Sysno::mmap => &[Hex, UInt, Hex, Hex, Int, Hex],
        Sysno::mprotect => &[Hex, UInt, Hex],
        Sysno::munmap => &[Hex, UInt],

        // process
        Sysno::arch_prctl => &[Hex, Hex],
        Sysno::exit_group => &[Int],
        Sysno::getrandom => &[Hex, UInt, Hex],
        Sysno::prlimit64 => &[Int, Int, Hex, Hex],
        Sysno::rseq => &[Hex, UInt, Hex, Hex],
        Sysno::set_robust_list => &[Hex, UInt],
        Sysno::set_tid_address => &[Hex],

        _ => return None,
    };

    Some(kinds)
}
