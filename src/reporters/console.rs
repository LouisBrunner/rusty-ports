use reporters;
use std::char;
use std::io;
use std::io::Write;
use std::sync::Mutex;

enum Writer<'a, T: io::Write> {
  File(Mutex<&'a mut T>),
  Stdout(io::Stdout),
  Stderr(io::Stderr),
}

pub struct ConsoleReporter<'a, T: io::Write> {
    out: Writer<'a, T>,
}

pub fn new_stdout(out: io::Stdout) -> impl reporters::Reporter {
    ConsoleReporter::<io::Stdout> { out: Writer::Stdout(out) }
}

pub fn new_stderr(out: io::Stderr) -> impl reporters::Reporter {
    ConsoleReporter::<io::Stderr> { out: Writer::Stderr(out) }
}

pub fn new_file<'a, T: io::Write>(file: Mutex<&'a mut T>) -> impl reporters::Reporter + 'a {
    ConsoleReporter { out: Writer::File(file) }
}

macro_rules! write_line {
    ($writer: expr, $header: ident, $gen: ident) => {
      write!(&mut $writer, "{} ", $header).expect("reporter: failed to write");
      $gen(&mut $writer).expect("reporter: failed to write");
    };
}

enum Tag {
    Monitor,
    Server { port: u16 },
    Client { id: usize },
}

impl<T: io::Write> ConsoleReporter<'_, T> {
    fn report(&self, tag: &Tag, msg: &str) {
        self.report_fn(tag, &|writer: &mut dyn io::Write| -> io::Result<()> {
            writeln!(writer, "{}", msg)?;
            Ok(())
        })
    }

    fn report_str(&self, tag: &Tag, msg: String) {
        self.report_fn(tag, &|writer: &mut dyn io::Write| -> io::Result<()> {
            writeln!(writer, "{}", msg)?;
            Ok(())
        })
    }

    fn report_fn(&self, tag: &Tag, gen: & dyn Fn(&mut dyn io::Write) -> io::Result<()>) {
        let header = match tag {
            Tag::Monitor => "[MONIT]".to_string(),
            Tag::Server { port } => format!("[SERVR][{}]", port),
            Tag::Client { id } => format!("[CLIEN][{}]", id),
        };

        match &self.out {
          Writer::Stdout(out) => { write_line!(out.lock(), header, gen); },
          Writer::Stderr(out) => { write_line!(out.lock(), header, gen); },
          Writer::File(file) => { write_line!(*file.lock().expect("failed to lock file"), header, gen); },
        }
    }
}

impl<T: io::Write> reporters::Reporter for ConsoleReporter<'_, T> {
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
        self.report_fn(&Tag::Client { id }, &|writer: &mut dyn io::Write| -> io::Result<()> {
            writeln!(writer, "Received message:")?;

            let chunk_size = 16;
            let chunk_groups = 4;
            for chunk in msg.chunks(chunk_size) {
                let len_diff = chunk_size - chunk.len();
                write!(writer, "\t")?;

                let mut i = 0;
                for c in chunk {
                    if i % chunk_groups == 0 && i != 0 {
                        write!(writer, " ")?;
                    }
                    write!(writer, "{:02x}", c)?;
                    i += 1
                }

                let padding_len = len_diff * 2 + (len_diff / chunk_groups);
                let padding = std::iter::repeat(" ").take(padding_len).collect::<String>();
                write!(writer, "{} |", padding)?;

                for c in chunk {
                    let cc = *c as char;
                    write!(writer, "{}", if char::is_control(cc) { '.' } else { cc })?;
                }

                let padding = std::iter::repeat(" ").take(len_diff).collect::<String>();
                writeln!(writer, "{}|", padding)?;
            }

            Ok(())
        });
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

#[cfg(test)]
mod tests {
    use mockstream::MockStream;
    use std::rc::Rc;
    use std::str;

    use super::*;
    use reporters::Reporter;

    fn expect_stream_contains(ms: Mutex<&mut MockStream>, expected: &str) {
        let written = ms
            .lock()
            .expect("could not read mock stream")
            .pop_bytes_written();
        assert_eq!(
            str::from_utf8(&written).expect("is not valid UTF-8"),
            expected
        );
    }

    #[test]
    fn it_notifies_start_stop() {
        let ms = &mut MockStream::new();
        let ms = Rc::new(Mutex::new(ms));
        let reporter = new_file(*ms.clone());
        let ms = *ms.clone();

        reporter.started();
        expect_stream_contains(ms, "[MONIT] Started\n");

        reporter.stopping();
        expect_stream_contains(ms, "[MONIT] Stopping...\n");

        reporter.stopped();
        expect_stream_contains(ms, "[MONIT] Stopped\n");
    }

    // #[test]
    // fn it_notifies_error_warning() {
    //     let ms = Mutex::new(&mut MockStream::new());
    //     let reporter = new_file(ms);

    //     reporter.error("123".to_owned());
    //     expect_stream_contains(ms, "[MONIT] Error: 123\n");

    //     reporter.warning("456".to_owned());
    //     expect_stream_contains(ms, "[MONIT] Warning: 456\n");
    // }

    // #[test]
    // fn it_notifies_server_changes() {
    //     let ms = Mutex::new(&mut MockStream::new());
    //     let reporter = new_file(ms);

    //     reporter.server_started(42);
    //     expect_stream_contains(ms, "[SERVR][42] Started\n");

    //     reporter.server_stopping(42);
    //     expect_stream_contains(ms, "[SERVR][42] Stopping...\n");

    //     reporter.server_stopped(42);
    //     expect_stream_contains(ms, "[SERVR][42] Stopped\n");
    // }

    // #[test]
    // fn it_notifies_client_changes() {
    //     let ms = Mutex::new(&mut MockStream::new());
    //     let reporter = new_file(ms);

    //     reporter.client_connected(1337, 42);
    //     expect_stream_contains(ms, "[CLIEN][1337] Connected on port 42\n");

    //     reporter.client_message_received(1337, &[1, 2, 3, 4, 5, 6, 56, 67, 78]);
    //     expect_stream_contains(
    //         ms,
    //         "[CLIEN][1337] Received message:\n\t01020304 05063843 4e                |......8CN       |\n",
    //     );

    //     reporter.client_message_received(1337, &[1, 2, 3, 4, 5, 6, 56, 67, 78, 1, 2, 3, 4, 5, 6, 7, 8]);
    //     expect_stream_contains(
    //         ms,
    //         "[CLIEN][1337] Received message:\n\t01020304 05063843 4e010203 04050607 |......8CN.......|\n\t08                                  |.               |\n",
    //     );

    //     reporter.client_disconnected(1337);
    //     expect_stream_contains(ms, "[CLIEN][1337] Disconnected\n");
    // }
}
