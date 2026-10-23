use syscalls::Sysno;

pub fn name(nr: u64) -> String {
    match Sysno::new(nr as usize) {
        Some(sysno) => sysno.name().to_string(),
        None => format!("syscall_{nr:#x}"),
    }
}
