#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use crate::link::Link;
use crate::markup::MarkupFile;
pub mod logger;
pub mod cli;
pub mod file_traversal;
pub mod link_extractor;
pub mod link_validator;
pub mod link;
pub mod markup;

#[derive(Default)]
pub struct Config {
    pub log_level: logger::LogLevel,
    pub folder: String,
    pub markup_types: Vec<markup::MarkupType>,
}

pub fn run(config: &Config) -> Result<(), &str> {
    println!("++++++++++++++++++++++++++++++++++");
    println!("++++++++++ linkchecker ++++++++++");
    println!("++++++++++++++++++++++++++++++++++");

    logger::init(&config.log_level);
    let mut files: Vec<MarkupFile> = Vec::new();
    file_traversal::find(&config, &mut files);

    let mut links: Vec<Link> = Vec::new();
    for file in files {
        links.append(&mut link_extractor::find_links(&file));
    }

    let mut result: Vec<Result<String, String>> = Vec::new();
    for link in links {
        result.push(link_validator::check(&link));
    }

    let mut invalid_links = false;
    for res in result {
        match res {
            Result::Ok(val) => debug!("{:?}", val),
            Result::Err(_) => invalid_links = true,
        }
    }

    if invalid_links { Err("Some links could not be resolved.") } else { Ok(()) }
}