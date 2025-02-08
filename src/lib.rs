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
use link_extractors::link_extractor::BrokenExtractedLink;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::vec;
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
    #[serde(rename(deserialize = "do-not-warn-for-redirect-to"))]
    pub do_not_warn_for_redirect_to: Option<Vec<String>>,
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
    #[serde(rename(deserialize = "gitignore"))]
    pub gitignore: Option<bool>,
    #[serde(rename(deserialize = "gituntracked"))]
    pub gituntracked: Option<bool>,
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
DoNotWarnForRedirectTo: {:?}
Types: {:?}
Offline: {}
MatchExt: {}
RootDir: {}
Gitignore: {}
Gituntracked: {}
IgnoreLinks: {}
IgnorePath: {:?}
Throttle: {} ms",
            self.optional.debug.unwrap_or(false),
            self.directory.to_str().unwrap_or_default(),
            self.optional.do_not_warn_for_redirect_to,
            markup_types_str,
            self.optional.offline.unwrap_or_default(),
            self.optional.match_file_extension.unwrap_or_default(),
            root_dir_str,
            self.optional.gitignore.unwrap_or_default(),
            self.optional.gituntracked.unwrap_or_default(),
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

fn find_all_links(config: &Config) -> Vec<Result<MarkupLink, BrokenExtractedLink>> {
    let mut files: Vec<MarkupFile> = Vec::new();
    file_traversal::find(config, &mut files);
    let mut links = vec![];
    for file in files {
        links.append(&mut link_extractors::link_extractor::find_links(&file));
    }
    links
}

fn find_git_ignored_files() -> Option<Vec<PathBuf>> {
    let output = Command::new("git")
        .arg("ls-files")
        .arg("--ignored")
        .arg("--others")
        .arg("--exclude-standard")
        .output()
        .expect("Failed to execute 'git' command");

    if output.status.success() {
        let ignored_files = String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 sequence")
            .lines()
            .filter(|line| line.ends_with(".md") || line.ends_with(".html"))
            .filter_map(|line| fs::canonicalize(Path::new(line.trim())).ok())
            .collect::<Vec<_>>();
        Some(ignored_files)
    } else {
        eprintln!(
            "git ls-files command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        None
    }
}
fn find_git_untracked_files() -> Option<Vec<PathBuf>> {
    let output = Command::new("git")
        .arg("ls-files")
        .arg("--others")
        .arg("--exclude-standard")
        .output()
        .expect("Failed to execute 'git' command");

    if output.status.success() {
        let ignored_files = String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 sequence")
            .lines()
            .filter(|line| line.ends_with(".md") || line.ends_with(".html"))
            .filter_map(|line| fs::canonicalize(Path::new(line.trim())).ok())
            .collect::<Vec<_>>();
        Some(ignored_files)
    } else {
        eprintln!(
            "git ls-files command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        None
    }
}

fn print_helper(
    link: &MarkupLink,
    status_code: &colored::ColoredString,
    msg: &str,
    error_channel: bool,
) {
    let mut link_str = format!("[{:^4}] {}", status_code, link.source_str());
    if !msg.is_empty() {
        link_str += &format!(" - {}", msg);
    }
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

    let gitignored_files: Option<Vec<PathBuf>> = if config.optional.gitignore.is_some() {
        let files = find_git_ignored_files();
        debug!("Found gitignored files: {:?}", files);
        files
    } else {
        None
    };

    let is_gitignore_enabled = gitignored_files.is_some();

    let gituntracked_files: Option<Vec<PathBuf>> = if config.optional.gituntracked.is_some() {
        let files = find_git_untracked_files();
        debug!("Found gituntracked files: {:?}", files);
        files
    } else {
        None
    };

    let is_gituntracked_enabled = gituntracked_files.is_some();

    let mut broken_references: Vec<BrokenExtractedLink> = vec![];
    for link in &links {
        match link {
            Ok(link) => {
                let canonical_link_source = match fs::canonicalize(&link.source) {
                    Ok(path) => path,
                    Err(e) => {
                        warn!(
                            "Failed to canonicalize link source: {}. Error: {:?}",
                            link.source, e
                        );
                        continue;
                    }
                };

                if is_gitignore_enabled {
                    if let Some(ref gif) = gitignored_files {
                        if gif.iter().any(|path| path == &canonical_link_source) {
                            print_helper(
                                link,
                                &"Skip".green(),
                                "Ignore link because it is ignored by git.",
                                false,
                            );
                            skipped += 1;
                            continue;
                        }
                    }
                }

                if is_gituntracked_enabled {
                    if let Some(ref gif) = gituntracked_files {
                        if gif.iter().any(|path| path == &canonical_link_source) {
                            print_helper(
                                link,
                                &"Skip".green(),
                                "Ignore link because it is untracked by git.",
                                false,
                            );
                            skipped += 1;
                            continue;
                        }
                    }
                }

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
            Err(broken_reference) => {
                broken_references.push(broken_reference.clone());
            }
        }
    }

    let do_not_warn_for_redirect_to: Arc<Vec<WildMatch>> =
        Arc::new(match &config.optional.do_not_warn_for_redirect_to {
            Some(s) => s.iter().map(|m| WildMatch::new(m)).collect(),
            None => vec![],
        });

    let throttle = config.optional.throttle.unwrap_or_default() > 0;
    info!("Throttle HTTP requests to same host: {:?}", throttle);
    let waits = Arc::new(Mutex::new(HashMap::new()));
    // See also http://patshaughnessy.net/2020/1/20/downloading-100000-files-using-async-rust
    let mut buffered_stream = stream::iter(link_target_groups.keys())
        .map(|target| {
            let waits = waits.clone();
            let do_not_warn_for_redirect_to = Arc::clone(&do_not_warn_for_redirect_to);
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

                let result_code = link_validator::check(
                    &target.target,
                    &target.link_type,
                    config,
                    &do_not_warn_for_redirect_to,
                )
                .await;

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
    if is_github_runner_env {
        info!("Running in github environment. Print errors and warnings as workflow commands");
    }

    let mut process_result = |result: FinalResult| match &result.result_code {
        LinkCheckResult::Ok => {
            oks += link_target_groups[&result.target].len();
        }
        LinkCheckResult::NotImplemented(msg) | LinkCheckResult::Warning(msg) => {
            warnings += link_target_groups[&result.target].len();
            if is_github_runner_env {
                for link in &link_target_groups[&result.target] {
                    println!(
                        "::warning file={},line={},col={},title=link checker warning::{}. {}",
                        link.source, link.line, link.column, result.target.target, msg
                    );
                }
            }
        }
        LinkCheckResult::Ignored(_) => {
            skipped += link_target_groups[&result.target].len();
        }
        LinkCheckResult::Failed(msg) => {
            errors.push(result.clone());
            if is_github_runner_env {
                for link in &link_target_groups[&result.target] {
                    println!(
                        "::error file={},line={},col={},title=broken link::{}. {}",
                        link.source, link.line, link.column, result.target.target, msg
                    );
                }
            }
        }
    };

    while let Some(result) = buffered_stream.next().await {
        print_result(&result, &link_target_groups);
        process_result(result);
    }
    for broken_ref in broken_references {
        warnings += 1;
        println!(
            "[{:^4}] {}:{}:{} => {} - {}",
            &"Warn".yellow(),
            broken_ref.source,
            broken_ref.line,
            broken_ref.column,
            broken_ref.reference,
            broken_ref.error
        );
        /* if is_github_runner_env {
            println!(
                "::warning file={},line={},col={},title=link checker warning::{}. {}",
                link.source, broken_reference., link.column, result.target.target, msg
            );
        } */
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
        println!();
        println!("The following links could not be resolved:");
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
