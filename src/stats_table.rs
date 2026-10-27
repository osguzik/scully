use std::collections::HashMap;
use std::time::Duration;

use crate::syscall_info;

const PERCENT_TIME_WIDTH: usize = 6;
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

        rows.sort_by(|(name_a, a), (name_b, b)| b.time.cmp(&a.time).then(name_a.cmp(name_b)));

        let sep = format!(
            "{} {} {} {} {} {}",
            "-".repeat(PERCENT_TIME_WIDTH),
            "-".repeat(TIME_WIDTH),
            "-".repeat(TIME_WIDTH),
            "-".repeat(COUNT_WIDTH),
            "-".repeat(COUNT_WIDTH),
            "-".repeat(NAME_WIDTH)
        );

        eprintln!(
            "{:>PERCENT_TIME_WIDTH$} {:>TIME_WIDTH$} {:>TIME_WIDTH$} {:>COUNT_WIDTH$} {:>COUNT_WIDTH$} syscall",
            "% time", "seconds", "usecs/call", "calls", "errors"
        );
        eprintln!("{sep}");

        let mut total_calls = 0;
        let mut total_errors = 0;

        let total_time: Duration = rows.iter().map(|(_, stats)| stats.time).sum();

        for (name, stats) in &rows {
            eprintln!(
                "{:>PERCENT_TIME_WIDTH$.2} {:>TIME_WIDTH$.6} {:>TIME_WIDTH$} {:>COUNT_WIDTH$} {:>COUNT_WIDTH$} {name}",
                time_percent(stats.time, total_time),
                stats.time.as_secs_f64(),
                stats.time.as_micros() / u128::from(stats.calls),
                stats.calls,
                errors_cell(stats.errors)
            );
            total_calls += stats.calls;
            total_errors += stats.errors;
        }

        eprintln!("{sep}");
        eprintln!(
            "{:>PERCENT_TIME_WIDTH$.2} {:>TIME_WIDTH$.6} {:>TIME_WIDTH$} {:>COUNT_WIDTH$} {:>COUNT_WIDTH$} total",
            100.0,
            total_time.as_secs_f64(),
            total_time.as_micros() / u128::from(total_calls),
            total_calls,
            errors_cell(total_errors)
        )
    }
}

fn time_percent(part: Duration, total: Duration) -> f64 {
    if total.is_zero() {
        0.0
    } else {
        part.as_secs_f64() / total.as_secs_f64() * 100.0
    }
}

fn errors_cell(errors: u64) -> String {
    if errors == 0 {
        String::new()
    } else {
        errors.to_string()
    }
}
