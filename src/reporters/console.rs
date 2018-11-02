use std::io;
use std::io::Write;
use reporters;
use itertools::Itertools;

pub struct ConsoleReporter {
}

pub fn new() -> ConsoleReporter {
    ConsoleReporter { }
}


enum Tag {
    Monitor,
    Server { port: u16 },
    Client { id: usize },
}

impl ConsoleReporter {
    fn report(&self, tag: &Tag, msg: &str) {
        let header =
            match tag {
                Tag::Monitor => "[MONIT]".to_string(),
                Tag::Server { port } => format!("[SERVR][{}]", port),
                Tag::Client { id } => format!("[CLIEN][{}]", id),
            };

        let stdout = io::stdout();
        writeln!(&mut stdout.lock(), "{}: {}", header, msg);
    }

    fn report_str(&self, tag: &Tag, msg: String) {
        self.report(tag, &*msg)
    }
}

impl reporters::Reporter for ConsoleReporter {
    fn started(&self) {
        self.report(&Tag::Monitor, "Started");
    }

    fn server_started(&self, port: u16) {
        self.report(&Tag::Server { port }, "Started");
    }

    fn client_connected(&self, id: usize, port: u16) {
        self.report_str(&Tag::Client { id }, format!("Connected on port {}", port));
    }

    fn client_message_received(&self, id: usize, msg: &[u8]) {
        self.report_str(&Tag::Client { id }, format!("Received message: {:02x}", msg.iter().format(" ")));
    }

    fn client_disconnected(&self, id: usize) {
        self.report(&Tag::Client { id }, "Disconnected");
    }

    fn server_stopping(&self, port: u16) {
        self.report(&Tag::Server { port }, "Stopping...");
    }

    fn server_stopped(&self, port: u16) {
        self.report(&Tag::Server { port }, "Stopped");
    }

    fn warning(&self, msg: String) {
        self.report_str(&Tag::Monitor, format!("Warning: {}", msg));
    }

    fn error(&self, msg: String) {
        self.report_str(&Tag::Monitor, format!("Error: {}", msg));
    }

    fn stopping(&self) {
        self.report(&Tag::Monitor, "Stopping...");
    }

    fn stopped(&self) {
        self.report(&Tag::Monitor, "Stopped");
    }
}
