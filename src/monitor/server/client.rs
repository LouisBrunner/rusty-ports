use crate::reporters::Reporter;

use async_std::{
    io::{Read, Result as IOResult},
    net::SocketAddr,
    net::TcpStream,
    prelude::*,
};
use std::{
    cmp::Ordering,
    sync::{Arc, Mutex},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("fatal error (IO={0}")]
    IO(#[from] std::io::Error),
}

#[cfg(test)]
use mockers_derive::mocked;

#[cfg_attr(test, mocked)]
pub trait Addressable {
    fn peer_addr(&self) -> IOResult<SocketAddr>;
}

impl Addressable for TcpStream {
    fn peer_addr(&self) -> IOResult<SocketAddr> {
        self.peer_addr()
    }
}

pub struct Client<T: Reporter, S: Read> {
    reporter: Arc<Mutex<T>>,
    port: u16,
    stream: S,
}

pub fn new<T: Reporter, S: Read>(reporter: Arc<Mutex<T>>, port: u16, stream: S) -> Client<T, S> {
    Client {
        reporter,
        port,
        stream,
    }
}

impl<T: Reporter, S: Read + Unpin + Addressable> Client<T, S> {
    pub async fn run(&mut self) -> Result<(), Error> {
        let addr = self.stream.peer_addr()?;
        let id = addr.port().into();

        self.reporter
            .lock()
            .unwrap()
            .client_connected(id, self.port);

        let mut buffer = [0u8; 1024];
        'outer: loop {
            let bytes = self.stream.read(&mut buffer).await?;
            match bytes.cmp(&0) {
                Ordering::Greater => {
                    self.reporter
                        .lock()
                        .unwrap()
                        .client_message_received(id, &buffer[..bytes]);
                }
                Ordering::Equal => {
                    break 'outer;
                }
                _ => (),
            }
        }

        self.reporter.lock().unwrap().client_disconnected(id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_std::{io::BufReader, task};
    use mockers::Scenario;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use super::*;

    pub struct FakeReader<'a> {
        content: BufReader<&'a [u8]>,
        addr: SocketAddr,
    }

    impl Read for FakeReader<'_> {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            _: &mut task::Context<'_>,
            buffer: &mut [u8],
        ) -> task::Poll<IOResult<usize>> {
            let result = task::block_on(async { self.content.read(buffer).await });
            task::Poll::Ready(result)
        }
    }

    impl Addressable for FakeReader<'_> {
        fn peer_addr(&self) -> IOResult<SocketAddr> {
            Ok(self.addr)
        }
    }

    #[test]
    fn it_handles_messages() {
        task::block_on(async {
            let scenario = Scenario::new();
            let (reporter, reporter_handle) = scenario.create_mock_for::<dyn Reporter>();
            let reader = FakeReader {
                content: BufReader::new("ABC 123".as_bytes()),
                addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1337),
            };

            scenario.expect(reporter_handle.client_connected(1337, 42).and_return(()));
            scenario.expect(
                reporter_handle
                    .client_message_received(1337, "ABC 123".as_bytes())
                    .and_return(()),
            );
            scenario.expect(reporter_handle.client_disconnected(1337).and_return(()));
            let mut client = new(Arc::new(Mutex::new(reporter)), 42, reader);
            assert!(client.run().await.is_ok());
        })
    }
}
