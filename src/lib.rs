#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use crate::link_extractors::link_extractor::MarkupLink;
use crate::markup::MarkupFile;
pub mod cli;
pub mod file_traversal;
pub mod link_extractors;
pub mod link_validator;
pub mod logger;
pub mod markup;
pub use colored::*;
pub use wildmatch::WildMatch;

use futures::{stream, StreamExt};
use link_validator::LinkCheckResult;

const PARALLEL_REQUESTS: usize = 20;

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

    println!("START");
    let link_check_results = stream::iter(links)
        .map(|link| {
            async move {
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
                    }
                }
                1
            }
        })
        .buffer_unordered(PARALLEL_REQUESTS);
    println!("END");

    link_check_results.for_each(|b|{
        async {
            println!("Bla");
        }
    }).await;
    /*
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
    }*/
    Ok(())
}
