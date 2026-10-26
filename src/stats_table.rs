use std::collections::HashMap;

use crate::syscall_info;

#[derive(Default)]
pub struct StatsTable {
    syscalls: HashMap<u64, u64>,
}

impl StatsTable {
    pub fn add_syscall(&mut self, nr: u64) {
        *self.syscalls.entry(nr).or_insert(0) += 1;
    }

    pub fn print_table(&self) {
        for (nr, count) in &self.syscalls {
            eprintln!("{count:>9} {}", syscall_info::name(*nr));
        }
    }
}
