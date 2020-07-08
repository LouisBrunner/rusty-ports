use std::sync::Arc;
use futures::future;
use futures::future::FutureExt;
use futures::select;
use async_std::task;

use crate::reporters::Reporter;

mod server;

pub use server::Error as Error;

pub struct Monitor<T: Reporter> {
    reporter: Arc<T>,
    start: u16,
    end: u16,
}

pub fn new<'a, T: Reporter>(reporter: T, start: u16, end: u16) -> Monitor<T> {
    Monitor {
        reporter: Arc::new(reporter),
        start,
        end,
    }
}

impl<T: Reporter> Monitor<T> {
    async fn exec<F: future::Future>(&self, interrupt: F) -> Result<(), server::Error> {
        let servers = (self.start..=self.end).map(|port: u16| -> server::Server<T> {
          server::new(self.reporter.clone(), port)
        }).collect::<Vec<_>>();

        let futures = servers.iter().map(|srv: &server::Server<T>| -> _ {
          srv.run()
        }).collect::<Vec<_>>();

        select! {
          res = future::try_join_all(futures).fuse() => res.map(|_| ()),
          _ = interrupt.fuse() => {
            self.reporter.stopping();
            Ok(())
          },
        }
    }

    pub fn start<F: future::Future>(&mut self, interrupt: F) -> Result<(), server::Error> {
        self.reporter.started();
        let result = task::block_on(self.exec(interrupt));
        self.reporter.stopped();
        result
    }
}

#[cfg(test)]
mod tests {
}
