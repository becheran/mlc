#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use std::error::Error;

// #[macro_use]
//extern crate log;
//extern crate simplelog;

pub mod logger;
pub mod cli;

pub struct Config {
    pub log_level: logger::LogLevel , // TODO
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    println!("++++++++++++++++++++++++++++++++++");
    println!("++++++++++ linkchecker ++++++++++");
    println!("++++++++++++++++++++++++++++++++++");

    logger::init(&config.log_level);

    Ok(())
}