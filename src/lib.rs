#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use crate::link::Link;
use crate::markup::MarkupFile;
pub mod cli;
pub mod file_traversal;
pub mod link;
pub mod link_extractor;
pub mod link_validator;
pub mod logger;
pub mod markup;
pub use colored::*;
use futures::executor::block_on;


#[derive(Default)]
pub struct Config {
    pub log_level: logger::LogLevel,
    pub folder: String,
    pub markup_types: Vec<markup::MarkupType>,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum LinkCheckResult {
    Ok(String),
    Failed(String),
    NotImplemented(String),
}

impl LinkCheckResult {
    pub fn success(&self) -> bool {
        match self {
            LinkCheckResult::Ok(..) => true,
            _ => false,
        }
    }
}

pub fn run(config: &Config) -> Result<(), ()> {
    println!("++++++++++++++++++++++++++++++++++");
    println!("++++++++++ linkchecker +++++++++++");
    println!("++++++++++++++++++++++++++++++++++");

    logger::init(&config.log_level);
    let mut files: Vec<MarkupFile> = Vec::new();
    file_traversal::find(&config, &mut files);

    let mut links: Vec<Link> = Vec::new();
    for file in files {
        links.append(&mut link_extractor::find_links(&file));
    }

    let mut result_futures= Vec::new();
    for link in &links {
        result_futures.push(link_validator::check(link));
    }
    let result = block_on(futures::future::join_all(result_futures));

    let mut invalid_links = vec!();
    let mut warnings = 0;
    for res in &result {
        match res {
            LinkCheckResult::Ok(val) => {
                println!("{} {}","OK".green(), val);
            }
            LinkCheckResult::NotImplemented(err) => {
                println!("{} {}","Warning".yellow(), err);
                warnings+=1;
            }
            LinkCheckResult::Failed(err) => {
                eprintln!("{} {}","Error".red(), err);
                invalid_links.push(err);
            }
        }
    }


    println!("");
    println!("Result");
    println!("");
     println!("Links: {}", &result.len());
    println!("Warnings: {}", warnings);
    println!("Errors: {}", &invalid_links.len());

    if !invalid_links.is_empty() {
        eprintln!("");
        eprintln!("The following links could not be resolved:");
        for il in invalid_links {
            eprintln!("   {} {}","Error".red(), il);
            
        }
        Err(())
    } else {
        Ok(())
    }
}
