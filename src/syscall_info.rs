use syscalls::Sysno;

#[derive(Clone, Copy)]
pub enum ArgKind {
    Hex,
    Str,
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
        Sysno::openat => &[Hex, Str, Hex, Hex],
        Sysno::newfstatat => &[Hex, Str, Hex, Hex],
        Sysno::access => &[Str, Hex],
        Sysno::unlinkat => &[Hex, Str, Hex],
        _ => return None,
    };

    Some(kinds)
}
