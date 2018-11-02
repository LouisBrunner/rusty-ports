use std::net;
use std::thread;
use std::time;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use reporters::Reporter;
use utils;

mod client;

pub struct Server<T: Reporter> {
    reporter: Arc<T>,
    running: Arc<AtomicBool>,
    port: u16,
}

pub fn new<T: Reporter>(reporter: Arc<T>, running: Arc<AtomicBool>, port: u16) -> Server<T> {
    Server { reporter, running, port }
}

impl<T: Reporter + Send + Sync + 'static> Server<T> {
    pub fn run(&self) -> bool {
        let listener = match net::TcpListener::bind(("0.0.0.0", self.port)) {
            Ok(server) => server,
            Err(err) => {
                self.reporter.error(format!("server(port: {}): {}", self.port, err.to_string()));
                return false;
            }
        };

        if let Err(err) = listener.set_nonblocking(true) {
            self.reporter.error(format!("server(port: {}): {}", self.port, err.to_string()));
            return false;
        }

        let running = Arc::new(AtomicBool::new(true));
        let mut children = vec![];

        self.reporter.server_started(self.port);

        while self.running.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let port = self.port;
                    let nreporter = self.reporter.clone();
                    let nrunning = running.clone();

                    children.push(thread::spawn(move || {
                        client::new(nreporter, nrunning, port, stream).run();
                    }));
                },
                Err(err) => {
                    if !utils::is_timeout_error(&err) {
                        self.reporter.error(format!("server(port: {}): {}", self.port, err.to_string()))
                    };
                },
            }

            // TODO: need a way to clean out the closed clients

            thread::sleep(time::Duration::from_millis(100));
        }

        self.reporter.server_stopping(self.port);
        running.store(false, Ordering::SeqCst);

        for child in children {
            child.join().expect("Could not join client thread");
        }

        self.reporter.server_stopped(self.port);

        true
    }
}
