use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use tokio::runtime::Runtime;
use tokio::net::TcpListener;
use tokio::prelude::Stream;
use tokio::prelude::Future;

use reporters::Reporter;

mod client;

pub struct Server<'a, T: Reporter> {
    reporter: &'a T,
    port: u16,
}

pub fn new<'a, T: Reporter>(reporter: &'a T, port: u16) -> Server<'a, T> {
    Server {
        reporter,
        port,
    }
}

impl<'a, T: Reporter> Server<'a, T> {
    pub fn run(&self, rt: &Runtime) -> Option<impl Future<Item=(), Error=()>> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), self.port);
        let listener = match TcpListener::bind(&addr) {
            Ok(server) => server,
            Err(err) => {
                self.reporter
                    .error(format!("server(port: {}): {}", self.port, err.to_string()));
                return None;
            }
        };

        self.reporter.server_started(self.port);

        let port = self.port;
        let port2 = self.port;
        let server = listener.incoming().for_each(move |socket| {
            client::new(self.reporter, port, socket).run(rt);
            Ok(())
        })
        .then(|res| -> Result<(), ()> {
            self.reporter.server_stopped(port2);
            res.map_err(|err| {
                self.reporter.error(format!("server(port: {}): {}", port2, err.to_string()));
                ()
            })
        })
        ;

        Some(server)
    }
}

#[cfg(test)]
mod tests {
    // use mockers::Scenario;
    // use std::thread;
    //
    // use super::*;
    //
    // #[test]
    // fn it_creates_a_nonblocking_server() {
    //     let scenario = Scenario::new();
    //     let reporter = Arc::new(scenario.create_mock_for::<Reporter>());
    //     let atomic = Arc::new(AtomicBool::new(true));
    //
    // TODO: fix
    //     thread::spawn(move || {
    //         let worked = new(reporter, atomic, 6666).run();
    //         assert!(worked);
    //     });
    //
    //     atomic.store(false, Ordering::SeqCst);
    // }
}
