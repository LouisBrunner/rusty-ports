use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::sync::Arc;
use std::time::Duration;

use net::RWTimeoutable;
use reporters::Reporter;
use utils;

static GLOBAL_CLIENT_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;

pub struct Client<T: Reporter, U: Read + RWTimeoutable> {
    reporter: Arc<T>,
    running: Arc<AtomicBool>,
    port: u16,
    stream: U,
}

pub fn new<T: Reporter, U: Read + RWTimeoutable>(
    reporter: Arc<T>,
    running: Arc<AtomicBool>,
    port: u16,
    stream: U,
) -> Client<T, U> {
    Client {
        reporter,
        running,
        port,
        stream,
    }
}

impl<T: Reporter + Send + Sync, U: Read + RWTimeoutable> Client<T, U> {
    pub fn run(&mut self) {
        let id = GLOBAL_CLIENT_COUNT.fetch_add(1, Ordering::SeqCst);

        if let Err(err) = self
            .stream
            .set_read_timeout(Some(Duration::from_millis(500)))
        {
            self.reporter.error(format!(
                "client(id: {}, pending: true): {}",
                id,
                err.to_string()
            ));
            return;
        }

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
                }
                Err(err) => {
                    if !utils::is_timeout_error(&err) {
                        self.reporter
                            .error(format!("client(id: {}): {}", id, err.to_string()))
                    }
                }
            }
        }

        self.reporter.client_disconnected(id);
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
