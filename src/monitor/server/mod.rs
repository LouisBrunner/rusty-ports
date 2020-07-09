use crate::reporters::Reporter;

mod client;

use async_std::{net::TcpListener, prelude::*, task};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("fatal error (client={0}")]
    Client(#[from] client::Error),
    #[error("fatal error (IO={0}")]
    IO(#[from] std::io::Error),
}

pub struct Server<T: Reporter> {
    reporter: Arc<Mutex<T>>,
    port: u16,
}

pub fn new<T: Reporter>(reporter: Arc<Mutex<T>>, port: u16) -> Server<T> {
    Server { reporter, port }
}

impl<T: Reporter + Send + 'static> Server<T> {
    pub async fn run(&self) -> Result<(), Error> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), self.port);
        let listener = TcpListener::bind(&addr).await?;
        let mut incoming = listener.incoming();

        self.reporter.lock().unwrap().server_started(self.port);

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            let reporter = self.reporter.clone();
            let port = self.port;
            task::spawn(async move {
                let err_reporter = reporter.clone();
                if let Err(e) = client::new(reporter, port, stream).run().await {
                    err_reporter
                        .lock()
                        .unwrap()
                        .error(format!("client failed: {}", e))
                }
            });
        }

        self.reporter.lock().unwrap().server_stopped(self.port);

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
