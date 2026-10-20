use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

use nix::sys::ptrace;

pub fn spawn_traced(cmd: &str, cmd_args: &[String]) -> io::Result<Child> {
    let mut command = Command::new(cmd);
    command.args(cmd_args);

    unsafe {
        command.pre_exec(|| ptrace::traceme().map_err(io::Error::from));
    }

    command.spawn()
}
