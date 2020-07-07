use tokio::runtime::Runtime;
use tokio::prelude::Future;
use futures::sync::oneshot;

use reporters::Reporter;

mod server;

pub struct Monitor<'a, T> {
    reporter: &'a T,
    start: u16,
    stop: u16,
    receiver: oneshot::Receiver<()>,
    sender: oneshot::Sender<()>,
}

pub fn new<'a, T: Reporter>(reporter: &'a T, start: u16, stop: u16) -> Monitor<'a, T> {
    let (sender, receiver) = oneshot::channel();

    Monitor {
        reporter,
        start,
        stop,
        sender,
        receiver,
    }
}

impl<'a, T: Reporter> Monitor<'a, T> {
    pub fn start(&self) -> bool {
        let mut rt = Runtime::new().expect("Could not start runtime");

        self.reporter.started();

        let mut futures = vec![];

        for port in self.start..=self.stop {
            match server::new(self.reporter, port).run(&rt) {
                Some(future) => futures.push(future),
                None => return false,
            }
        }

        let joined_futures = join_all(&futures);

        let stopper = self.receiver.map(|_| {
            self.reporter.stopping();
            Ok(())
        });
        joined_futures.select(stopper).wait().expect("Could not run servers");

        rt.shutdown_now().wait().expect("Could not stop servers");

        self.reporter.stopped();

        true
    }

    pub fn stopper(&self) -> impl Fn() + 'static {
        return move || {
        };
    }
}

fn join_all<I, E>(futures: &[&Future<Item=I, Error=E>]) -> Box<Future<Item=I, Error=E>> {
    match futures {
        [x] => Box::new(x),
        // &[x, y, ref rest..] => join_all([x.join(y), rest..]),
        _ => unimplemented!(),
    }
}

// TODO: basic tests
