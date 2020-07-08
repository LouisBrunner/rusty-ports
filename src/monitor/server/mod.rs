// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
// use tokio::runtime::Runtime;
// use tokio::net::TcpListener;
// use tokio::prelude::Stream;
// use tokio::prelude::future::Future;
use std::time::Duration;
use std::sync::Arc;
use async_std::task;
use thiserror::Error;

use crate::reporters::Reporter;

// mod client;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid toolchain name: {name}")]
    InvalidToolchainName {
        name: String,
    },
    #[error("unknown toolchain version: {version}")]
    UnknownToolchainVersion {
        version: String,
    }
}

pub struct Server<T: Reporter> {
    reporter: Arc<T>,
    port: u16,
}

pub fn new<T: Reporter>(reporter: Arc<T>, port: u16) -> Server<T> {
    Server {
        reporter,
        port,
    }
}

impl<T: Reporter> Server<T> {
    pub async fn run(&self) -> Result<(), Error> {
        loop {
            self.reporter.error(format!("server(port: {}): hello!", self.port));
            task::sleep(Duration::from_secs(1)).await;
        }
        // let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), self.port);
        // let listener = match TcpListener::bind(&addr) {
        //     Ok(server) => server,
        //     Err(err) => {
        //         self.reporter
        //             .error(format!("server(port: {}): {}", self.port, err.to_string()));
        //         return false;
        //     }
        // };

        // self.reporter.server_started(self.port);

        // let port = self.port;
        // let server = listener.incoming().for_each(move |socket| {
        //     client::new(self.reporter, port, socket).run(rt);
        //     Ok(())
        // })
        // .then(|res| -> Result<(), ()> {
        //     self.reporter.server_stopped(port);
        //     res.map_err(|err| {
        //         self.reporter.error(format!("server(port: {}): {}", port, err.to_string()));
        //         ()
        //     })
        // })
        // ;

        // true
    }
}

#[cfg(test)]
mod tests {
}
