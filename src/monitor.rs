use std::io::Read;
use std::net;
use std::thread;

extern crate itertools;
use monitor::itertools::Itertools;

pub struct Monitor {
    start: u16,
    stop: u16,
}

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

fn start_server(port: u16) {
    let listener = net::TcpListener::bind(("0.0.0.0", port)).unwrap();

    for client in listener.incoming() {
        match client {
            Ok(stream) => new_client(port, &stream),
            Err(err) => println!("error in server for port {}: {}", port, err),
        }
    }
}

fn new_client(port: u16, mut stream: &net::TcpStream) {
    let mut buffer = [0u8; 1024];
    println!("client connected on port {}", port);
    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    return;
                } else {
                    println!("client on port {}: {:02x}", port, buffer[..n].iter().format(" "))
                }
            }
            Err(_) => return,
        }
    }
}
