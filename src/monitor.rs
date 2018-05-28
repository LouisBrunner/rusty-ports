pub struct Monitor {
    start: u16,
    stop: u16,
}

use std::thread;

impl Monitor {
    pub fn new(start: u16, stop: u16) -> Monitor {
        Monitor { start, stop }
    }

    pub fn start(&self) {
        let mut children = vec![];

        for port in self.start..=self.stop {
            children.push(thread::spawn(move || {
                start_server(port);
            }));
        }

        for child in children {
            let _ = child.join();
        }
    }
}

use std::net;

fn start_server(port: u16) {
    let listener = net::TcpListener::bind(("0.0.0.0", port)).unwrap();

    for client in listener.incoming() {
        match client {
            Ok(mut stream) => new_client(port, &mut stream),
            Err(err) => println!("error in server for port {}: {}", port, err),
        }
    }
}

use std::io::Read;
use itertools::Itertools;

fn new_client(port: u16, stream: &mut Read) {
    let mut buffer = [0u8; 1024];
    println!("client connected on port {}", port);
    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    return;
                } else {
                    println!(
                        "client on port {}: {:02x}",
                        port,
                        buffer[..n].iter().format(" ")
                    )
                }
            }
            Err(err) => println!("error in client for port {}: {}", port, err),
        }
    }
}

#[cfg(test)]
mod tests {
    use mockstream::MockStream;

    use monitor::new_client;

    #[test]
    fn it_handles_client() {
        let mut ms = MockStream::new();
        new_client(42, &mut ms);
    }
}
