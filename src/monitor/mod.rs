// use tokio::runtime::Runtime;
// use tokio::prelude::future::Future;
// use futures::sync::oneshot;
use futures::future;
use async_std::task;

use crate::reporters::Reporter;

mod server;

pub struct Monitor<'a, T: Reporter> {
    reporter: &'a T,
    start: u16,
    stop: u16,
}

pub fn new<'a, T: Reporter>(reporter: &'a T, start: u16, stop: u16) -> Monitor<'a, T> {
    Monitor {
        reporter,
        start,
        stop,
    }
}

impl<'a, T: Reporter> Monitor<'a, T> {
    async fn exec(&self) -> Vec<bool> {
        let servers = (self.start..=self.stop).map(|port: u16| -> server::Server<T> {
          server::new(self.reporter, port)
        }).collect::<Vec<_>>();

        let futures = servers.iter().map(|srv: &server::Server<T>| -> _ {
          srv.run()
        }).collect::<Vec<_>>();

        future::join_all(futures).await
    }

    pub fn start(&self) -> bool {
        self.reporter.started();

        let results = task::block_on(self.exec());

        self.reporter.stopped();

        results.iter().fold(true, |acc: bool, i: &bool| {
          acc && *i
        })
    }

    pub fn stop(&self) {
        self.reporter.stopping()
        // TODO?
    }
}

#[cfg(test)]
mod tests {
}
