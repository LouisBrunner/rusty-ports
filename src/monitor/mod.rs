use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use reporters::Reporter;

mod server;

pub enum Message {
    ServerStopped { port: u16 },
    Stop,
}

pub struct Monitor<T> {
    reporter: Arc<T>,
    start: u16,
    stop: u16,
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,

}

pub fn new<T: Reporter>(reporter: T, start: u16, stop: u16) -> Monitor<T> {
    let (sender, receiver) = mpsc::channel();

    Monitor { reporter: Arc::new(reporter), start, stop, sender, receiver }
}

impl<T: Reporter + Send + Sync + 'static> Monitor<T> {
    pub fn start(&self) {
        let mut children = vec![];
        let running = Arc::new(AtomicBool::new(true));

        self.reporter.started();

        for port in self.start..=self.stop {
            let nreporter = self.reporter.clone();
            let nrunning = running.clone();
            let tx = self.sender.clone();

            children.push(thread::spawn(move || {
                let requested = server::new(nreporter, nrunning, port).run();
                if !requested {
                    tx.send(Message::ServerStopped { port }).expect("Could not stop program (internal failure)");
                }
            }));
        }

        match self.receiver.recv() {
            Ok(msg) => match msg {
                Message::ServerStopped { .. } => (),
                Message::Stop => (),
            },
            Err(err) => self.reporter.error(err.to_string()),
        }

        self.reporter.stopping();
        running.store(false, Ordering::SeqCst);

        for child in children {
            child.join().expect("Could not join server thread");
        }

        self.reporter.stopped();
    }

    pub fn sender(&self) -> mpsc::Sender<Message> {
        self.sender.clone()
    }
}
