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
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt;
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

#[derive(Default, Debug, Deserialize)]
pub struct OptionalConfig {
    pub debug: Option<bool>,
    #[serde(rename(deserialize = "markup-types"))]
    pub markup_types: Option<Vec<markup::MarkupType>>,
    pub offline: Option<bool>,
    #[serde(rename(deserialize = "match-file-extension"))]
    pub match_file_extension: Option<bool>,
    #[serde(rename(deserialize = "ignore-links"))]
    pub ignore_links: Option<Vec<String>>,
    #[serde(rename(deserialize = "ignore-path"))]
    pub ignore_path: Option<Vec<PathBuf>>,
    #[serde(rename(deserialize = "root-dir"))]
    pub root_dir: Option<PathBuf>,
    pub throttle: Option<u32>,
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub directory: PathBuf,
    pub optional: OptionalConfig,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ignore_str: Vec<String> = match &self.optional.ignore_links {
            Some(s) => s.iter().map(|m| m.to_string()).collect(),
            None => vec![],
        };
        let root_dir_str = match &self.optional.root_dir {
            Some(p) => p.to_str().unwrap_or(""),
            None => "",
        };
        let ignore_path_str: Vec<String> = match &self.optional.ignore_path {
            Some(p) => p.iter().map(|m| m.to_str().unwrap().to_string()).collect(),
            None => vec![],
        };
        let markup_types_str: Vec<String> = match &self.optional.markup_types {
            Some(p) => p.iter().map(|m| format!["{:?}", m]).collect(),
            None => vec![],
        };
        write!(
            f,
            "
Debug: {:?}
Dir: {} 
Types: {:?} 
Offline: {}
MatchExt: {}
RootDir: {}
IgnoreLinks: {} 
IgnorePath: {:?}
Throttle: {} ms",
            self.optional.debug.unwrap_or(false),
            self.directory.to_str().unwrap_or_default(),
            markup_types_str,
            self.optional.offline.unwrap_or_default(),
            self.optional.match_file_extension.unwrap_or_default(),
            root_dir_str,
            ignore_str.join(","),
            ignore_path_str,
            self.optional.throttle.unwrap_or(0)
        )
    }
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
    let link_str = format!("[{:^4}] {} - {}", status_code, link.source_str(), msg);
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

    let ignore_links: Vec<WildMatch> = match &config.optional.ignore_links {
        Some(s) => s.iter().map(|m| WildMatch::new(m)).collect(),
        None => vec![],
    };
    for link in &links {
        if ignore_links.iter().any(|m| m.matches(&link.target)) {
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

    let throttle = config.optional.throttle.unwrap_or_default() > 0;
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
                            *old + Duration::from_millis(
                                config.optional.throttle.unwrap_or_default().into(),
                            )
                        }
                        None => {
                            Instant::now()
                                + Duration::from_millis(
                                    config.optional.throttle.unwrap_or_default().into(),
                                )
                        }
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

    let is_github_runner_env = env::var("GITHUB_ENV").is_ok();
    if is_github_runner_env{
        info!("Running in github environment. Print errors and warnings as workflow commands");
    }

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
            LinkCheckResult::Failed(err) => {
                errors.push(result.clone());     
                if is_github_runner_env{       
                    for link in &link_target_groups[&result.target] {
                        println!("::error file={},line={},col={},title=broken link::Target {}. {}", link.source, link.line, link.column, result.target.target, err);    
                    }
                }
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
                println!("{}", link.source_str());
            }
        }
        println!();
        Err(())
    }
}
