use std::collections::HashMap;

use crate::syscall_info;

#[derive(Default)]
struct SyscallStats {
    calls: u64,
    errors: u64,
}

#[derive(Default)]
pub struct StatsTable {
    syscalls: HashMap<u64, SyscallStats>,
}

impl StatsTable {
    pub fn add_syscall(&mut self, nr: u64, ret: i64) {
        let stats = self.syscalls.entry(nr).or_default();
        stats.calls += 1;
        if syscall_info::is_error(ret) {
            stats.errors += 1;
        }
    }

    pub fn print_table(&self) {
        let mut rows: Vec<(String, &SyscallStats)> = self
            .syscalls
            .iter()
            .map(|(nr, stats)| (syscall_info::name(*nr), stats))
            .collect();

        rows.sort_by(|(name_a, a), (name_b, b)| b.calls.cmp(&a.calls).then(name_a.cmp(name_b)));

        for (name, stats) in &rows {
            eprintln!("{:>9} {:>9} {name}", stats.calls, stats.errors);
        }
    }
}
