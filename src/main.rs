extern crate ctrlc;
#[cfg(test)]
extern crate mockers;
#[cfg(test)]
extern crate mockers_derive;
#[cfg(test)]
extern crate testing_logger;

#[macro_use]
extern crate clap;
extern crate users;
extern crate log;
extern crate simplelog;
extern crate tokio;
extern crate futures;

use std::process;

use clap::{App, Arg};
use users::get_current_uid;

mod monitor;
// mod net;
mod reporters;

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
    simplelog::TermLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Stdout).unwrap();

    let app = get_app();
    let app_help = app.clone();

    let matches = app.get_matches();

    let start = value_t_or_exit!(matches, "RANGE_START", u16);
    let end = value_t_or_exit!(matches, "RANGE_END", u16);

    if start > end {
        app_help.clone().print_help().expect("failed to show help");
        process::exit(1);
    }

    if start <= 1024 && get_current_uid() != 0 {
        fatal_error("must be root to use ports from 1 to 1024");
    }

    let reporter = reporters::console::new();

    let monitor = monitor::new(&reporter, start, end);
    ctrlc::set_handler(|| {
      monitor.stopper()
    }).expect("Error setting Ctrl-C handler");

    if !monitor.start() {
        process::exit(1);
    }
}
