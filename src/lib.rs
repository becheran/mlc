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
    fn print_helper(
        link: &MarkupLink,
        status_code: &colored::ColoredString,
        msg: &str,
        error_channel: bool,
    ) {
        let link_str = format!(
            "[{:^4}] {} ({}, {}) => {}. {}",
            status_code, link.source, link.line, link.column, link.target, msg
        );
        if error_channel {
            eprintln!("{}", link_str);
        } else {
            println!("{}", link_str);
        }
    }

    match &result.result_code {
        LinkCheckResult::Ok => {
            print_helper(&result.link, &"OK".green(), "", false);
        }
        LinkCheckResult::NotImplemented(msg) => {
            print_helper(&result.link, &"Warn".yellow(), msg, false);
        }
        LinkCheckResult::Warning(msg) => {
            print_helper(&result.link, &"Warn".yellow(), msg, false);
        }
        LinkCheckResult::Ignored(msg) => {
            print_helper(&result.link, &"Skip".green(), msg, false);
        }
        LinkCheckResult::Failed(msg) => {
            print_helper(&result.link, &"Err".red(), msg, true);
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

    let mut skipped = vec![];
    let mut errors = vec![];
    let mut warnings = vec![];
    let mut oks = vec![];
    while let Some(result) = link_check_results.next().await {
        print_result(&result);
        match &result.result_code {
            LinkCheckResult::Ok => {
                oks.push(result.clone());
            }
            LinkCheckResult::NotImplemented(_) | LinkCheckResult::Warning(_) => {
                warnings.push(result.clone());
            }
            LinkCheckResult::Ignored(_) => {
                skipped.push(result.clone());
            }
            LinkCheckResult::Failed(_) => {
                errors.push(result.clone());
            }
        }
    }

    println!();
    let sum = skipped.len() + errors.len() + warnings.len() + oks.len();
    println!("Result ({} links):", sum);
    println!();
    println!("OK       {}", oks.len());
    println!("Skipped  {}", skipped.len());
    println!("Warnings {}", warnings.len());
    println!("Errors   {}", errors.len());
    println!();

    if errors.len() > 0 {
        eprintln!();
        eprintln!("The following links could not be resolved:");
        println!();
        for res in errors {
            let error_msg = format!(
                "{} ({}, {}) => {}.",
                res.link.source, res.link.line, res.link.column, res.link.target
            );
            eprintln!("{}", error_msg);
        }
        println!();
        Err(())
    } else {
        Ok(())
    }
}
