use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs::OpenOptions;

use sunfish::uci::uci_loop;

fn set_global_logger() {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("sunfish_log.log")
        .expect("Can't create log file in directory");
    let _ = WriteLogger::init(LevelFilter::Trace, Config::default(), file);
}

fn main() {
    set_global_logger();
    uci_loop();
}
