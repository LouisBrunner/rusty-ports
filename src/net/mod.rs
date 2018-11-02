use std::io;
use std::net::TcpStream;
use std::time::Duration;

pub trait RWTimeoutable {
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()>;
    fn read_timeout(&self) -> io::Result<Option<Duration>>;

    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()>;
    fn write_timeout(&self) -> io::Result<Option<Duration>>;
}

impl RWTimeoutable for TcpStream {
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        TcpStream::set_read_timeout(self, dur)
    }

    fn read_timeout(&self) -> io::Result<Option<Duration>> {
        TcpStream::read_timeout(self)
    }

    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        TcpStream::set_write_timeout(self, dur)
    }

    fn write_timeout(&self) -> io::Result<Option<Duration>> {
        TcpStream::write_timeout(self)
    }
}
