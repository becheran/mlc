#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use crate::link_extractors::link_extractor::MarkupLink;
use crate::link_validator::link_type::get_link_type;
use crate::link_validator::link_type::LinkType;
use crate::link_validator::resolve_target_link;
use crate::markup::MarkupFile;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep_until, Duration, Instant};
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
use url::Url;

const PARALLEL_REQUESTS: usize = 20;

#[derive(Default, Debug)]
pub struct Config {
    pub log_level: logger::LogLevel,
    pub folder: PathBuf,
    pub markup_types: Vec<markup::MarkupType>,
    pub no_web_links: bool,
    pub match_file_extension: bool,
    pub ignore_links: Vec<WildMatch>,
    pub ignore_path: Vec<PathBuf>,
    pub root_dir: Option<PathBuf>,
    pub throttle: u32,
}

#[derive(Debug, Clone)]
struct FinalResult {
    target: Target,
    result_code: LinkCheckResult,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
struct Target {
    target: String,
    link_type: LinkType,
}

fn find_all_links(config: &Config) -> Vec<MarkupLink> {
    let mut files: Vec<MarkupFile> = Vec::new();
    file_traversal::find(config, &mut files);
    let mut links = vec![];
    for file in files {
        links.append(&mut link_extractors::link_extractor::find_links(&file));
    }
    links
}

fn print_helper(
    link: &MarkupLink,
    status_code: &colored::ColoredString,
    msg: &str,
    error_channel: bool,
) {
    let link_str = format!(
        "[{:^4}] {} ({}, {}) => {} - {}",
        status_code, link.source, link.line, link.column, link.target, msg
    );
    if error_channel {
        eprintln!("{}", link_str);
    } else {
        println!("{}", link_str);
    }
}

fn print_result(result: &FinalResult, map: &HashMap<Target, Vec<MarkupLink>>) {
    for link in &map[&result.target] {
        match &result.result_code {
            LinkCheckResult::Ok => {
                print_helper(link, &"OK".green(), "", false);
            }
            LinkCheckResult::NotImplemented(msg) | LinkCheckResult::Warning(msg) => {
                print_helper(link, &"Warn".yellow(), msg, false);
            }
            LinkCheckResult::Ignored(msg) => {
                print_helper(link, &"Skip".green(), msg, false);
            }
            LinkCheckResult::Failed(msg) => {
                print_helper(link, &"Err".red(), msg, true);
            }
        }
    }
}

pub async fn run(config: &Config) -> Result<(), ()> {
    let links = find_all_links(config);
    let mut link_target_groups: HashMap<Target, Vec<MarkupLink>> = HashMap::new();

    let mut skipped = 0;

    for link in &links {
        if config.ignore_links.iter().any(|m| m.matches(&link.target)) {
            print_helper(
                link,
                &"Skip".green(),
                "Ignore link because of ignore-links option.",
                false,
            );
            skipped += 1;
            continue;
        }
        let link_type = get_link_type(&link.target);
        let target = resolve_target_link(link, &link_type, config).await;
        let t = Target { target, link_type };
        match link_target_groups.get_mut(&t) {
            Some(v) => v.push(link.clone()),
            None => {
                link_target_groups.insert(t, vec![link.clone()]);
            }
        }
    }

    let throttle = config.throttle > 0;
    info!("Throttle HTTP requests to same host: {:?}", throttle);
    let waits = Arc::new(Mutex::new(HashMap::new()));
    // See also http://patshaughnessy.net/2020/1/20/downloading-100000-files-using-async-rust
    let mut buffered_stream = stream::iter(link_target_groups.keys())
        .map(|target| {
            let waits = waits.clone();
            async move {
                if throttle && target.link_type == LinkType::Http {
                    let parsed = match Url::parse(&target.target) {
                        Ok(parsed) => parsed,
                        Err(error) => {
                            return FinalResult {
                                target: target.clone(),
                                result_code: LinkCheckResult::Failed(format!(
                                    "Could not parse URL type. Err: {:?}",
                                    error
                                )),
                            }
                        }
                    };
                    let host = match parsed.host_str() {
                        Some(host) => host.to_string(),
                        None => {
                            return FinalResult {
                                target: target.clone(),
                                result_code: LinkCheckResult::Failed(
                                    "Failed to determine host".to_string(),
                                ),
                            }
                        }
                    };
                    let mut waits = waits.lock().await;

                    let mut wait_until: Option<Instant> = None;
                    let next_wait = match waits.get(&host) {
                        Some(old) => {
                            wait_until = Some(*old);
                            *old + Duration::from_millis(config.throttle.into())
                        }
                        None => Instant::now() + Duration::from_millis(config.throttle.into()),
                    };
                    waits.insert(host, next_wait);
                    drop(waits);

                    if let Some(deadline) = wait_until {
                        sleep_until(deadline).await;
                    }
                }

                let result_code =
                    link_validator::check(&target.target, &target.link_type, config).await;

                FinalResult {
                    target: target.clone(),
                    result_code,
                }
            }
        })
        .buffer_unordered(PARALLEL_REQUESTS);

    let mut oks = 0;
    let mut warnings = 0;
    let mut errors = vec![];

    let mut process_result = |result| {
        print_result(&result, &link_target_groups);
        match &result.result_code {
            LinkCheckResult::Ok => {
                oks += link_target_groups[&result.target].len();
            }
            LinkCheckResult::NotImplemented(_) | LinkCheckResult::Warning(_) => {
                warnings += link_target_groups[&result.target].len();
            }
            LinkCheckResult::Ignored(_) => {
                skipped += link_target_groups[&result.target].len();
            }
            LinkCheckResult::Failed(_) => {
                errors.push(result.clone());
            }
        }
    };

    while let Some(result) = buffered_stream.next().await {
        process_result(result);
    }

    println!();
    let error_sum: usize = errors
        .iter()
        .map(|e| link_target_groups[&e.target].len())
        .sum();
    let sum = skipped + error_sum + warnings + oks;
    println!("Result ({} links):", sum);
    println!();
    println!("OK       {}", oks);
    println!("Skipped  {}", skipped);
    println!("Warnings {}", warnings);
    println!("Errors   {}", error_sum);
    println!();

    if errors.is_empty() {
        Ok(())
    } else {
        eprintln!();
        eprintln!("The following links could not be resolved:");
        println!();
        for res in errors {
            for link in &link_target_groups[&res.target] {
                eprintln!(
                    "{} ({}, {}) => {}.",
                    link.source, link.line, link.column, link.target
                );
            }
        }
        println!();
        Err(())
    }
}
