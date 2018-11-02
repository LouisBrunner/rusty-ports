use std::io;

pub fn is_timeout_error(err: &io::Error) -> bool {
    err.kind() == std::io::ErrorKind::TimedOut || err.kind() == std::io::ErrorKind::WouldBlock
}
