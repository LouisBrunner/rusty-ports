use std::io::Read;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

use net::RWTimeoutable;
use reporters::Reporter;
use utils;

static GLOBAL_CLIENT_COUNT : AtomicUsize = ATOMIC_USIZE_INIT;

pub struct Client<T: Reporter, U: Read + RWTimeoutable> {
    reporter: Arc<T>,
    running: Arc<AtomicBool>,
    port: u16,
    stream: U,
}

pub fn new<T: Reporter, U: Read + RWTimeoutable>(reporter: Arc<T>, running: Arc<AtomicBool>, port: u16, stream: U) -> Client<T, U> {
    Client { reporter, running, port, stream }
}

impl<T: Reporter + Send + Sync, U: Read + RWTimeoutable> Client<T, U> {
    pub fn run(&mut self) {
        let id = GLOBAL_CLIENT_COUNT.fetch_add(1, Ordering::SeqCst);

        match self.stream.set_read_timeout(Some(Duration::from_millis(500))) {
            Ok(_) => (),
            Err(err) => {
                self.reporter.error(format!("client(id: {}, pending: true): {}", id, err.to_string()));
                return;
            },
        };

        self.reporter.client_connected(id, self.port);

        let mut buffer = [0u8; 1024];
        while self.running.load(Ordering::SeqCst) {
            match self.stream.read(&mut buffer) {
                Ok(n) => {
                    if n < 1 {
                        break;
                    } else {
                        self.reporter.client_message_received(id, &buffer[..n]);
                    }
                },
                Err(err) =>  {
                    if !utils::is_timeout_error(&err) {
                        self.reporter.error(format!("client(id: {}): {}", id, err.to_string()))
                    }
                },
            }
        }

        self.reporter.client_disconnected(id);
    }
}

// TODO: fix tests
#[cfg(test)]
mod tests {
    use mockstream::MockStream;

    // use monitor::server::client;

    #[test]
    fn it_handles_client() {
        let mut ms = MockStream::new();
        new(42, &mut ms);
    }
}
