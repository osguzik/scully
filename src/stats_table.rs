use std::collections::HashMap;

use crate::syscall_info;

const COUNT_WIDTH: usize = 9;
const NAME_WIDTH: usize = 16;

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

        let sep = format!(
            "{} {} {}",
            "-".repeat(COUNT_WIDTH),
            "-".repeat(COUNT_WIDTH),
            "-".repeat(NAME_WIDTH)
        );

        eprintln!(
            "{:>COUNT_WIDTH$} {:>COUNT_WIDTH$} syscall",
            "calls", "errors"
        );
        eprintln!("{sep}");

        let mut total_calls = 0;
        let mut total_errors = 0;

        for (name, stats) in &rows {
            eprintln!(
                "{:>COUNT_WIDTH$} {:>COUNT_WIDTH$} {name}",
                stats.calls,
                errors_cell(stats.errors)
            );
            total_calls += stats.calls;
            total_errors += stats.errors;
        }

        eprintln!("{sep}");
        eprintln!(
            "{:>COUNT_WIDTH$} {:>COUNT_WIDTH$} total",
            total_calls,
            errors_cell(total_errors)
        )
    }
}

fn errors_cell(errors: u64) -> String {
    if errors == 0 {
        String::new()
    } else {
        errors.to_string()
    }
}
