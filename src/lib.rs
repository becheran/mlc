use crate::checker::{Checker, Target};
use crate::config::wildcards;
use crate::link_extractors::link_extractor::{BrokenExtractedLink, MarkupLink};
use crate::link_validator::link_type::get_link_type;
use crate::link_validator::resolve_target_link;
use crate::printer::Printer;
use crate::report::{Issue, Report, Severity, Summary};
use futures::{StreamExt, stream};
use link_validator::LinkCheckResult;
use log::{debug, info, warn};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

mod checker;
pub mod cli;
mod config;
pub mod file_traversal;
mod git;
pub mod link_extractors;
pub mod link_validator;
pub mod logger;
pub mod markup;
mod printer;
pub mod report;

pub use config::{Config, OptionalConfig};
pub use wildmatch::WildMatch;

const PARALLEL_REQUESTS: usize = 20;

/// Why [`run`] did not succeed
#[derive(Debug)]
pub enum Error {
    /// At least one link is broken. The broken links have already been reported.
    BrokenLinks,
    /// The CSV report could not be written
    CsvFile { path: PathBuf, source: io::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BrokenLinks => write!(f, "Found broken links"),
            Error::CsvFile { path, source } => {
                write!(f, "Failed to write CSV file {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::BrokenLinks => None,
            Error::CsvFile { source, .. } => Some(source),
        }
    }
}

fn find_all_links(config: &Config) -> Vec<Result<MarkupLink, BrokenExtractedLink>> {
    let mut files = Vec::new();
    file_traversal::find(config, &mut files);
    files
        .iter()
        .flat_map(link_extractors::link_extractor::find_links)
        .collect()
}

/// Reasons to not check a link
struct SkipRules {
    gitignored: Option<HashSet<PathBuf>>,
    untracked: Option<HashSet<PathBuf>>,
    ignore_links: Vec<WildMatch>,
}

impl SkipRules {
    fn new(config: &Config) -> Self {
        let optional = &config.optional;
        let gitignored = optional
            .gitignore
            .unwrap_or_default()
            .then(|| git::ignored_files(config))
            .flatten();
        debug!("Found gitignored files: {gitignored:?}");
        let untracked = optional
            .gituntracked
            .unwrap_or_default()
            .then(|| git::untracked_files(config))
            .flatten();
        debug!("Found gituntracked files: {untracked:?}");

        SkipRules {
            gitignored,
            untracked,
            ignore_links: wildcards(optional.ignore_links.as_deref()),
        }
    }

    /// Why the link in the canonical `source` file is not checked, if it is not
    fn reason(&self, source: &Path, link: &MarkupLink) -> Option<&'static str> {
        if self.gitignored.as_ref().is_some_and(|f| f.contains(source)) {
            Some("Ignore link because it is ignored by git.")
        } else if self.untracked.as_ref().is_some_and(|f| f.contains(source)) {
            Some("Ignore link because it is untracked by git.")
        } else if self.ignore_links.iter().any(|m| m.matches(&link.target)) {
            Some("Ignore link because of ignore-links option.")
        } else {
            None
        }
    }
}

pub async fn run(config: &Config) -> Result<(), Error> {
    let links = find_all_links(config);
    let printer = Printer::new(&config.optional);
    let skip_rules = SkipRules::new(config);

    let mut summary = Summary::default();
    let mut issues: Vec<Issue> = vec![];

    // All links which point to the same target are checked together
    let mut link_target_groups: HashMap<Target, Vec<MarkupLink>> = HashMap::new();
    let mut broken_references: Vec<&BrokenExtractedLink> = vec![];
    // Links of one file are consecutive, so remembering the last source avoids
    // canonicalizing the same path for every single link.
    let mut last_source: Option<(&str, Option<PathBuf>)> = None;
    for link in &links {
        let link = match link {
            Ok(link) => link,
            Err(broken_reference) => {
                broken_references.push(broken_reference);
                continue;
            }
        };

        if last_source
            .as_ref()
            .is_none_or(|(source, _)| *source != link.source)
        {
            let canonical = fs::canonicalize(&link.source)
                .inspect_err(|e| {
                    warn!(
                        "Failed to canonicalize link source: {}. Error: {e:?}",
                        link.source
                    );
                })
                .ok();
            last_source = Some((&link.source, canonical));
        }
        let Some((_, Some(canonical_link_source))) = &last_source else {
            continue;
        };

        if let Some(reason) = skip_rules.reason(canonical_link_source, link) {
            printer.skipped(link, reason);
            summary.skipped += 1;
            continue;
        }

        let link_type = get_link_type(&link.target);
        let target = resolve_target_link(link, &link_type, config, Some(canonical_link_source));
        link_target_groups
            .entry(Target { target, link_type })
            .or_default()
            .push(link.clone());
    }

    if printer.github_annotations() {
        info!("Running in github environment. Print errors and warnings as workflow commands");
    }
    let checker = Checker::new(config);
    // See also http://patshaughnessy.net/2020/1/20/downloading-100000-files-using-async-rust
    let mut checked_targets = stream::iter(link_target_groups.keys())
        .map(|target| checker.check(target))
        .buffer_unordered(PARALLEL_REQUESTS);

    while let Some(checked) = checked_targets.next().await {
        for link in &link_target_groups[checked.target] {
            printer.result(link, &checked.result);
            printer.annotation(link, &checked.target.target, &checked.result);
            match &checked.result {
                LinkCheckResult::Ok => summary.ok += 1,
                LinkCheckResult::Ignored(_) => summary.skipped += 1,
                LinkCheckResult::NotImplemented(msg) | LinkCheckResult::Warning(msg) => {
                    summary.warnings += 1;
                    issues.push(Issue::from_link(Severity::Warning, link, msg));
                }
                LinkCheckResult::Failed(msg) => {
                    summary.errors += 1;
                    issues.push(Issue::from_link(Severity::Error, link, msg));
                }
            }
        }
    }

    for broken_reference in broken_references {
        printer.broken_reference(broken_reference);
        summary.warnings += 1;
        issues.push(Issue::from_broken_reference(broken_reference));
    }

    let report = Report::new(summary, issues);
    printer.report(&report);

    if let Some(path) = &config.optional.csv_file {
        info!("Write CSV file: {}", path.display());
        report.write_csv(path).map_err(|source| Error::CsvFile {
            path: path.clone(),
            source,
        })?;
    }

    if report.summary.errors == 0 {
        Ok(())
    } else {
        Err(Error::BrokenLinks)
    }
}
