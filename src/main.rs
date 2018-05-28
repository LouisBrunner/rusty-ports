use std::process;

extern crate itertools;
extern crate mockstream;

#[macro_use]
extern crate clap;
use clap::{App, Arg};

extern crate users;
use users::get_current_uid;

mod monitor;

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

fn main() {
    let app = get_app();

    let matches = app.get_matches();

    let start = value_t!(matches, "RANGE_START", u16).unwrap();
    let end = value_t!(matches, "RANGE_END", u16).unwrap();

    if start > end {
        get_app().print_help().expect("failed to show help");
        println!();
        process::exit(1);
    }

    if start <= 1024 {
        if get_current_uid() != 0 {
            println!("must be root to use ports from 1 to 1024");
            process::exit(1);
        }
    }

    monitor::Monitor::new(start, end).start();
}
