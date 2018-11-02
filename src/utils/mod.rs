use std::io;

pub fn is_timeout_error(err: &io::Error) -> bool {
    err.kind() == std::io::ErrorKind::TimedOut || err.kind() == std::io::ErrorKind::WouldBlock
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_a_timeout() {
        assert!(is_timeout_error(&io::Error::from(io::ErrorKind::TimedOut)));
        assert!(is_timeout_error(&io::Error::from(
            io::ErrorKind::WouldBlock
        )));
        assert!(!is_timeout_error(&io::Error::from(io::ErrorKind::NotFound)));
        assert!(!is_timeout_error(&io::Error::from(
            io::ErrorKind::ConnectionAborted
        )));
    }
}
