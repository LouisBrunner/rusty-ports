extern crate ctrlc;
#[cfg(test)]
extern crate mockers;
#[cfg(test)]
extern crate mockers_derive;
#[cfg(test)]
extern crate mockstream;
#[macro_use]
extern crate clap;
extern crate users;

use std::io;
use std::process;
use std::sync::{Arc, Mutex};

use clap::{App, Arg};
use users::get_current_uid;

mod monitor;
mod net;
mod reporters;
mod utils;

fn get_app<'a>() -> App<'a, 'a> {
    App::new("rusty-ports")
        .about("Monitor a range of ports")
        .arg(Arg::with_name("RANGE_START")
             .help("The start of the range of ports to monitor")
             .required(true)
             .index(1))
        .arg(Arg::with_name("RANGE_END")
             .help("The end of the range of ports to monitor (must be greater than or equal to RANGE_START, inclusive)")
             .required(true)
             .index(2))
}

fn fatal_error(error: &str) {
    println!("{}", error);
    process::exit(1);
}

fn main() {
    let app = get_app();
    let app_help = app.clone();

    let matches = app.get_matches();

    let start = value_t!(matches, "RANGE_START", u16).expect("failed to get the range start");
    let end = value_t!(matches, "RANGE_END", u16).expect("failed to get the range end");

    if start > end {
        app_help.clone().print_help().expect("failed to show help");
        fatal_error("");
    }

    if start <= 1024 && get_current_uid() != 0 {
        fatal_error("must be root to use ports from 1 to 1024");
    }

    let stdout = io::stdout();
    let reporter = reporters::console::new(Arc::new(Mutex::new(stdout)));

    let monitor = monitor::new(reporter, start, end);
    let sender = monitor.sender();

    ctrlc::set_handler(move || {
        sender
            .send(monitor::Message::Stop)
            .expect("Could not stop program");
    })
    .expect("Error setting Ctrl-C handler");

    monitor.start();
}
