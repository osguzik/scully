use nix::sys::ptrace;
use nix::unistd::Pid;

/// Linux only accepts path arguments up to PATH_MAX bytes, including the terminating zero.
const PATH_MAX: usize = 4096;

pub fn read_string(pid: Pid, addr: u64) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut addr = addr;

    while bytes.len() < PATH_MAX {
        let word = ptrace::read(pid, addr as ptrace::AddressType).ok()?;

        for byte in word.to_ne_bytes() {
            if byte == 0 {
                return Some(bytes);
            }
            bytes.push(byte);
        }

        addr += 8;
    }

    Some(bytes)
}
