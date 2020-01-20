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

#[derive(Debug, Clone)]
struct FinalResult {
    link: MarkupLink,
    result_code: LinkCheckResult,
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

fn print_result(result: &FinalResult) {
    match &result.result_code {
        LinkCheckResult::Ok => {
            println!(
                "[{:^4}] {} ({}, {}) => {}",
                "OK".green(),
                result.link.source,
                result.link.line,
                result.link.column,
                &result.link.target
            );
        }
        LinkCheckResult::NotImplemented(msg) => {
            println!(
                "[{:^4}] {} ({}, {}) => {}. {}",
                "Warn".yellow(),
                result.link.source,
                result.link.line,
                result.link.column,
                result.link.target,
                msg
            );
        }
        LinkCheckResult::Warning(msg) => {
            println!(
                "[{:^4}] {} ({}, {}) => {}. {}",
                "Warn".yellow(),
                result.link.source,
                result.link.line,
                result.link.column,
                result.link.target,
                msg
            );
        }
        LinkCheckResult::Ignored(msg) => {
            println!(
                "[{:^4}] {} ({}, {}) => {}. {}",
                "Skip".green(),
                result.link.source,
                result.link.line,
                result.link.column,
                result.link.target,
                msg
            );
        }
        LinkCheckResult::Failed(msg) => {
            let error_msg = format!(
                "[{:^4}] {} ({}, {}) => {}. {}",
                "Err".red(),
                result.link.source,
                result.link.line,
                result.link.column,
                result.link.target,
                msg
            );
            eprintln!("{}", &error_msg);
        }
    }
}

pub async fn run(config: &Config) -> Result<(), ()> {
    let links = find_all_links(&config);

    let mut link_check_results = stream::iter(links)
        .map(|link| {
            async move {
                let result_code = link_validator::check(&link.source, &link.target, &config).await;
                FinalResult {
                    link: link,
                    result_code: result_code,
                }
            }
        })
        .buffer_unordered(PARALLEL_REQUESTS);

    let mut final_result = vec![];
    while let Some(result) = link_check_results.next().await {
        final_result.push(result.clone());
        print_result(&result);
    }

    println!();
    println!("Result ({} links):", final_result.len());
    println!();
    println!(
        "OK       {}",
        final_result
            .iter()
            .filter(|x| match x.result_code {
                LinkCheckResult::Ok => true,
                _ => false,
            })
            .count()
    );
    println!(
        "Skipped  {}",
        final_result
            .iter()
            .filter(|x| match x.result_code {
                LinkCheckResult::Ignored(_) => true,
                _ => false,
            })
            .count()
    );
    println!(
        "Warnings {}",
        final_result
            .iter()
            .filter(|x| match x.result_code {
                LinkCheckResult::Warning(_) => true,
                _ => false,
            })
            .count()
    );
    println!(
        "Errors   {}",
        final_result
            .iter()
            .filter(|x| match x.result_code {
                LinkCheckResult::Failed(_) => true,
                _ => false,
            })
            .count()
    );
    println!();

    if final_result
        .iter()
        .filter(|x| match x.result_code {
            LinkCheckResult::Failed(_) => true,
            _ => false,
        })
        .count()
        > 0
    {
        eprintln!();
        eprintln!("The following links could not be resolved:");
        println!();
        for res in final_result.iter().filter(|x| match x.result_code {
            LinkCheckResult::Failed(_) => true,
            _ => false,
        }) {
            eprintln!("{:?}", res.link);
        }
        println!();
        Err(())
    } else {
        Ok(())
    }
}
