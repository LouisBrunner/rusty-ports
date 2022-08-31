use crate::reporters;

use std::{fmt, fmt::Write};

use log::{error, info, warn};

pub struct ConsoleReporter {}

pub fn new() -> impl reporters::Reporter {
    ConsoleReporter {}
}

enum Tag {
    Monitor,
    Server { port: u16 },
    Client { id: usize },
}

impl ConsoleReporter {
    fn report<S: Into<String>>(&self, tag: &Tag, msg: S) {
        let header = match tag {
            Tag::Monitor => "[MONIT]".to_string(),
            Tag::Server { port } => format!("[SERVR][{}]", port),
            Tag::Client { id } => format!("[CLIEN][{}]", id),
        };

        info!("{} {}", header, msg.into());
    }
}

fn format_hex(msg: &[u8]) -> Result<String, fmt::Error> {
    let mut s = String::new();

    writeln!(s, "Received message:")?;

    let chunk_size = 16;
    let chunk_groups = 4;
    for chunk in msg.chunks(chunk_size) {
        let len_diff = chunk_size - chunk.len();
        write!(s, "\t")?;

        for (i, c) in chunk.iter().enumerate() {
            if i % chunk_groups == 0 && i != 0 {
                write!(s, " ")?;
            }
            write!(s, "{:02x}", c)?;
        }

        let padding_len = len_diff * 2 + (len_diff / chunk_groups);
        let padding = " ".repeat(padding_len);
        write!(s, "{} |", padding)?;

        for c in chunk {
            let cc = *c as char;
            write!(s, "{}", if char::is_control(cc) { '.' } else { cc })?;
        }

        let padding = " ".repeat(len_diff);
        writeln!(s, "{}|", padding)?;
    }

    Ok(s)
}

impl reporters::Reporter for ConsoleReporter {
    fn started(&self) {
        self.report(&Tag::Monitor, "Started");
    }

    fn server_started(&self, port: u16) {
        self.report(&Tag::Server { port }, "Started");
    }

    fn client_connected(&self, id: usize, port: u16) {
        self.report(&Tag::Client { id }, format!("Connected on port {}", port));
    }

    fn client_message_received(&self, id: usize, msg: &[u8]) {
        self.report(
            &Tag::Client { id },
            &format_hex(msg).expect("reporter: could not generate"),
        );
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
        warn!("{}", msg);
    }

    fn error(&self, msg: String) {
        error!("{}", msg);
    }

    fn stopping(&self) {
        self.report(&Tag::Monitor, "Stopping...");
    }

    fn stopped(&self) {
        self.report(&Tag::Monitor, "Stopped");
    }
}

#[cfg(test)]
mod tests {
    use log;
    use std::str;
    use testing_logger;

    use super::*;
    use reporters::Reporter;

    fn expect_log_contains(expected: &str, level: log::Level) {
        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 1);
            assert_eq!(captured_logs[0].body, expected);
            assert_eq!(captured_logs[0].level, level);
        });
    }

    #[test]
    fn it_notifies_start_stop() {
        testing_logger::setup();
        let reporter = new();

        reporter.started();
        expect_log_contains("[MONIT] Started", log::Level::Info);

        reporter.stopping();
        expect_log_contains("[MONIT] Stopping...", log::Level::Info);

        reporter.stopped();
        expect_log_contains("[MONIT] Stopped", log::Level::Info);
    }

    #[test]
    fn it_notifies_error_warning() {
        testing_logger::setup();
        let reporter = new();

        reporter.error("123".to_owned());
        expect_log_contains("123", log::Level::Error);

        reporter.warning("456".to_owned());
        expect_log_contains("456", log::Level::Warn);
    }

    #[test]
    fn it_notifies_server_changes() {
        testing_logger::setup();
        let reporter = new();

        reporter.server_started(42);
        expect_log_contains("[SERVR][42] Started", log::Level::Info);

        reporter.server_stopping(42);
        expect_log_contains("[SERVR][42] Stopping...", log::Level::Info);

        reporter.server_stopped(42);
        expect_log_contains("[SERVR][42] Stopped", log::Level::Info);
    }

    #[test]
    fn it_notifies_client_changes() {
        testing_logger::setup();
        let reporter = new();

        reporter.client_connected(1337, 42);
        expect_log_contains("[CLIEN][1337] Connected on port 42", log::Level::Info);

        reporter.client_message_received(1337, &[1, 2, 3, 4, 5, 6, 56, 67, 78]);
        expect_log_contains(
            "[CLIEN][1337] Received message:\n\t01020304 05063843 4e                |......8CN       |\n",
            log::Level::Info
        );

        reporter.client_message_received(
            1337,
            &[1, 2, 3, 4, 5, 6, 56, 67, 78, 1, 2, 3, 4, 5, 6, 7, 8],
        );
        expect_log_contains(
            "[CLIEN][1337] Received message:\n\t01020304 05063843 4e010203 04050607 |......8CN.......|\n\t08                                  |.               |\n",
            log::Level::Info
        );

        reporter.client_disconnected(1337);
        expect_log_contains("[CLIEN][1337] Disconnected", log::Level::Info);
    }
}
