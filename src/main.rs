#[cfg(test)] extern crate mockers;
#[cfg(test)] extern crate mockers_derive;
#[cfg(test)] extern crate testing_logger;

mod monitor;
// mod net;
mod reporters;

#[macro_use] extern crate clap;
use clap::{App, Arg};

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

use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("must be root to use ports from 1 to 1024")]
    PrivilegedPorts,
    #[error("RANGE_START must be lesser or equal to RANGE_END")]
    WrongRange,
    #[error("monitoring failed ({0})")]
    MonitorError(#[from] monitor::Error),
    #[error("fatal error (clap={0})")]
    Clap(#[from] clap::Error),
}

use users::get_current_uid;
use futures::future;

fn inner_main() -> Result<(), AppError> {
    simplelog::TermLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default(), simplelog::TerminalMode::Stdout).unwrap();

    let app = get_app();
    let app_help = app.clone();

    let matches = app.get_matches();

    let start = value_t!(matches, "RANGE_START", u16)?;
    let end = value_t!(matches, "RANGE_END", u16)?;

    if start > end {
        app_help.clone().print_help()?;
        print!("\n\n");
        return Err(AppError::WrongRange);
    }

    if start <= 1024 && get_current_uid() != 0 {
        return Err(AppError::PrivilegedPorts);
    }

    let reporter = reporters::console::new();
    let mut monitor = monitor::new(reporter, start, end);

    // TODO: add SIGINT catching

    monitor.start::<future::Pending<()>>(future::pending())?;

    Ok(())
}

fn main() {
  use std::process::exit;
  use libc::{EXIT_SUCCESS, EXIT_FAILURE};

  // FIXME: rework this once Termination lands in stable (next decade)
  exit(match inner_main() {
      Ok(_) => EXIT_SUCCESS,

      Err(ref err) => {
          println!("error: {}", err);
          EXIT_FAILURE
      }
  })
}
