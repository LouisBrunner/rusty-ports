use crate::reporters::Reporter;

use thiserror::Error;
use async_std::{net::TcpStream, prelude::*};
use std::sync::{Arc, Mutex};

#[derive(Error, Debug)]
pub enum Error {
    #[error("fatal error (IO={0}")]
    IO(#[from] std::io::Error),
}

pub struct Client<T: Reporter> {
    reporter: Arc<Mutex<T>>,
    port: u16,
    stream: TcpStream,
}

pub fn new<T: Reporter>(reporter: Arc<Mutex<T>>, port: u16, stream: TcpStream) -> Client<T> {
    Client {
        reporter,
        port,
        stream,
    }
}

impl<T: Reporter> Client<T> {
    pub async fn run(&mut self) -> Result<(), Error> {
        let addr = self.stream.peer_addr()?;
        let id = addr.port().into();

        self.reporter.lock().unwrap().client_connected(id, self.port);

        let mut buffer = [0u8; 1024];
        loop {
            let bytes = self.stream.read(&mut buffer).await?;
            if bytes > 0 {
                self.reporter.lock().unwrap().client_message_received(id, &buffer[..bytes]);
            } else if bytes == 0 {
                break;
            }
        }

        self.reporter.lock().unwrap().client_disconnected(id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use mockstream::MockStream;
    // use mockers::Scenario;
    // use std::thread;
    //
    // use super::*;
    //
    // #[test]
    // fn it_handles_messages() {
    //     let scenario = Scenario::new();
    //     let reporter = Arc::new(scenario.create_mock_for::<Reporter>());
    //     let ms = MockStream::new();
    //     let atomic = Arc::new(AtomicBool::new(true));
    //
    //     // TODO: finish and more cases
    //     // thread::spawn(move || {
    //     //     new(reporter, atomic, 42, ms).run();
    //     // });
    //
    //     atomic.store(false, Ordering::SeqCst);
    // }
}
