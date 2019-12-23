#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use crate::markup::MarkupFile;
pub mod cli;
pub mod file_traversal;
pub mod link_extractors;
pub mod link_validator;
pub mod logger;
pub mod markup;
pub use colored::*;

use link_validator::LinkCheckResult;

#[derive(Default)]
pub struct Config {
    pub log_level: logger::LogLevel,
    pub folder: String,
    pub markup_types: Vec<markup::MarkupType>,
}

pub fn run(config: &Config) -> Result<(), ()> {
    let mut files: Vec<MarkupFile> = Vec::new();
    file_traversal::find(&config, &mut files);

    let mut warnings = 0;
    let mut link_ctr = 0;
    let mut invalid_links = vec![];
    for file in files {
        let links = link_extractors::link_extractor::find_links(&file);
        for link in links {
            link_ctr += 1;
            let result = link_validator::check(&file.path, &link.target);
            match result {
                LinkCheckResult::Ok => {
                    println!("{} {:?}", "OK".green(), link);
                }
                LinkCheckResult::NotImplemented(msg) => {
                    println!("{} {:?} {}", "Warning".yellow(), link, msg);
                    warnings += 1;
                }
                LinkCheckResult::Warning(msg) => {
                    println!("{} {:?} {}", "Warning".yellow(), link, msg);
                    warnings += 1;
                }
                LinkCheckResult::Failed(msg) => {
                    eprintln!("{} {:?} {}", "Error".red(), link, msg);
                    invalid_links.push(link);
                }
            }
        }
    }

    println!();
    println!("Result:");
    println!();
    println!("Links: {}", link_ctr);
    println!("Warnings: {}", warnings);
    println!("Errors: {}", &invalid_links.len());
    println!();

    if !invalid_links.is_empty() {
        eprintln!();
        eprintln!("The following links could not be resolved:");
        println!();
        for il in invalid_links {
            eprintln!("{} {:?}", "Error".red(), il);
        }
        Err(())
    } else {
        Ok(())
    }
}
