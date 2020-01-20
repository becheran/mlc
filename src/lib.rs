#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use crate::markup::MarkupFile;
use crate::link_extractors::link_extractor::MarkupLink;
pub mod cli;
pub mod file_traversal;
pub mod link_extractors;
pub mod link_validator;
pub mod logger;
pub mod markup;
pub use colored::*;
pub use wildmatch::WildMatch;

use link_validator::LinkCheckResult;

#[derive(Default, Debug)]
pub struct Config {
    pub log_level: logger::LogLevel,
    pub folder: String,
    pub markup_types: Vec<markup::MarkupType>,
    pub no_web_links: bool,
    pub ignore_links: Vec<WildMatch>,
}

fn find_all_links(config: &Config) -> Vec<MarkupLink> {
    let mut files: Vec<MarkupFile> = Vec::new();
    file_traversal::find(&config, &mut files);
    let mut links = vec![];
    for file in files {
        links.append(&mut link_extractors::link_extractor::find_links(&file));
    }
    links
}

pub async fn run(config: &Config) -> Result<(), ()> {
    let links = find_all_links(&config);

    let mut invalid_links = vec![];
    let mut warnings_ctr = 0;
    let mut skipped_ctr = 0;
    let mut ok_ctr = 0;
    let mut link_ctr = 0;
    for link in links {
        link_ctr += 1;
        let result = link_validator::check(&link.source, &link.target, &config).await;
        match result {
            LinkCheckResult::Ok => {
                println!(
                    "[{:^4}] {} ({}, {}) => {}",
                    "OK".green(),
                    link.source,
                    link.line,
                    link.column,
                    link.target
                );
                ok_ctr += 1;
            }
            LinkCheckResult::NotImplemented(msg) => {
                println!(
                    "[{:^4}] {} ({}, {}) => {}. {}",
                    "Warn".yellow(),
                    link.source,
                    link.line,
                    link.column,
                    link.target,
                    msg
                );
                warnings_ctr += 1;
            }
            LinkCheckResult::Warning(msg) => {
                println!(
                    "[{:^4}] {} ({}, {}) => {}. {}",
                    "Warn".yellow(),
                    link.source,
                    link.line,
                    link.column,
                    link.target,
                    msg
                );
                warnings_ctr += 1;
            }
            LinkCheckResult::Ignored(msg) => {
                println!(
                    "[{:^4}] {} ({}, {}) => {}. {}",
                    "Skip".green(),
                    link.source,
                    link.line,
                    link.column,
                    link.target,
                    msg
                );
                skipped_ctr += 1;
            }
            LinkCheckResult::Failed(msg) => {
                let error_msg = format!(
                    "[{:^4}] {} ({}, {}) => {}. {}",
                    "Err".red(),
                    link.source,
                    link.line,
                    link.column,
                    link.target,
                    msg
                );
                eprintln!("{}", &error_msg);
                invalid_links.push(error_msg);
            }
        }
    }

    println!();
    println!("Result ({} links):", link_ctr);
    println!();
    println!("OK       {}", ok_ctr);
    println!("Skipped  {}", skipped_ctr);
    println!("Warnings {}", warnings_ctr);
    println!("Errors   {}", &invalid_links.len());
    println!();

    if !invalid_links.is_empty() {
        eprintln!();
        eprintln!("The following links could not be resolved:");
        println!();
        for il in invalid_links {
            eprintln!("{}", il);
        }
        println!();
        Err(())
    } else {
        Ok(())
    }
}
