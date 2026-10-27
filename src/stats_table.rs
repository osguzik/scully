use std::collections::HashMap;
use std::time::Duration;

use crate::syscall_info;

const TIME_WIDTH: usize = 11;
const COUNT_WIDTH: usize = 9;
const NAME_WIDTH: usize = 16;

#[derive(Default)]
struct SyscallStats {
    calls: u64,
    errors: u64,
    time: Duration,
}

#[derive(Default)]
pub struct StatsTable {
    syscalls: HashMap<u64, SyscallStats>,
}

impl StatsTable {
    pub fn add_syscall(&mut self, nr: u64, ret: i64, time: Duration) {
        let stats = self.syscalls.entry(nr).or_default();
        stats.calls += 1;
        stats.time += time;
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
            "{} {} {} {} {}",
            "-".repeat(TIME_WIDTH),
            "-".repeat(TIME_WIDTH),
            "-".repeat(COUNT_WIDTH),
            "-".repeat(COUNT_WIDTH),
            "-".repeat(NAME_WIDTH)
        );

        eprintln!(
            "{:>TIME_WIDTH$} {:>TIME_WIDTH$} {:>COUNT_WIDTH$} {:>COUNT_WIDTH$} syscall",
            "seconds", "usecs/call", "calls", "errors"
        );
        eprintln!("{sep}");

        let mut total_calls = 0;
        let mut total_errors = 0;
        let mut total_time = Duration::ZERO;

        for (name, stats) in &rows {
            eprintln!(
                "{:>TIME_WIDTH$.6} {:>TIME_WIDTH$} {:>COUNT_WIDTH$} {:>COUNT_WIDTH$} {name}",
                stats.time.as_secs_f64(),
                stats.time.as_micros() / u128::from(stats.calls),
                stats.calls,
                errors_cell(stats.errors)
            );
            total_calls += stats.calls;
            total_errors += stats.errors;
            total_time += stats.time;
        }

        eprintln!("{sep}");
        eprintln!(
            "{:>TIME_WIDTH$.6} {:>TIME_WIDTH$} {:>COUNT_WIDTH$} {:>COUNT_WIDTH$} total",
            total_time.as_secs_f64(),
            total_time.as_micros() / u128::from(total_calls),
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
