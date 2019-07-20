use crate::Config;
use crate::logger;
use clap::App;


pub fn parse_args() -> Config {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let debug = matches.is_present("debug");
    let log_level = if debug {logger::LogLevel::Debug} else {logger::LogLevel::Warn};
    debug!("The log level is: {}", log_level);
    Config { log_level }
}