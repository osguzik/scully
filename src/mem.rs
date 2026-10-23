use nix::sys::ptrace;
use nix::unistd::Pid;

pub fn read_string(pid: Pid, addr: u64) -> Option<Vec<u8>> {
    let word = ptrace::read(pid, addr as ptrace::AddressType).ok()?;

    let mut bytes = Vec::new();
    for byte in word.to_ne_bytes() {
        if byte == 0 {
            break;
        }
        bytes.push(byte);
    }

    Some(bytes)
}
