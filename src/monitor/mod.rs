use crate::reporters::Reporter;

mod server;

use async_std::task;
use futures::{future, future::FutureExt, select};
use std::sync::{Arc, Mutex};

pub use server::Error;

pub struct Monitor<T: Reporter> {
    reporter: Arc<Mutex<T>>,
    start: u16,
    end: u16,
}

pub fn new<T: Reporter>(reporter: Arc<Mutex<T>>, start: u16, end: u16) -> Monitor<T> {
    Monitor {
        reporter,
        start,
        end,
    }
}

impl<T: Reporter + Send + 'static> Monitor<T> {
    async fn exec<F: future::Future>(&self, interrupt: F) -> Result<(), server::Error> {
        let servers = (self.start..=self.end)
            .map(|port: u16| -> server::Server<T> { server::new(self.reporter.clone(), port) })
            .collect::<Vec<_>>();

        let futures = servers
            .iter()
            .map(|srv: &server::Server<T>| -> _ { srv.run() })
            .collect::<Vec<_>>();

        select! {
          res = future::try_join_all(futures).fuse() => res.map(|_| ()),
          _ = interrupt.fuse() => {
            self.reporter.lock().unwrap().stopping();
            Ok(())
          },
        }
    }

    pub fn start<F: future::Future>(&self, interrupt: F) -> Result<(), server::Error> {
        self.reporter.lock().unwrap().started();
        let result = task::block_on(self.exec(interrupt));
        self.reporter.lock().unwrap().stopped();
        result
    }
}

#[cfg(test)]
mod tests {}
