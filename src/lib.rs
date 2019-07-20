#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use std::error::Error;

pub mod logger;
pub mod cli;
pub mod file_traversal;

pub struct Config {
    pub log_level: logger::LogLevel,
    pub folder: String,
    pub file_extensions: Vec<String>
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    println!("++++++++++++++++++++++++++++++++++");
    println!("++++++++++ linkchecker ++++++++++");
    println!("++++++++++++++++++++++++++++++++++");

    logger::init(&config.log_level);
    let mut files: Vec<String> = Vec::new();
    file_traversal::find(&config.folder, &config.file_extensions, &mut files);

    Ok(())
}