use std::io;
use std::time::Duration;
use std;
use tokio;

pub trait RWTimeoutable {
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()>;
    fn read_timeout(&self) -> io::Result<Option<Duration>>;

    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()>;
    fn write_timeout(&self) -> io::Result<Option<Duration>>;
}

macro_rules! mimpl {
    ($T:ty => $($t:path),+) => {
        $(impl $T for $t {
            fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
                Self::set_read_timeout(self, dur)
            }

            fn read_timeout(&self) -> io::Result<Option<Duration>> {
                Self::read_timeout(self)
            }

            fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
                Self::set_write_timeout(self, dur)
            }

            fn write_timeout(&self) -> io::Result<Option<Duration>> {
                Self::write_timeout(self)
            }
        })*
    }
}


mimpl!{
    RWTimeoutable => std::net::TcpStream, tokio::net::TcpStream
}
