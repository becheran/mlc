//! Everything which is written to the terminal while links are checked

use crate::config::OptionalConfig;
use crate::link_extractors::link_extractor::{BrokenExtractedLink, MarkupLink, format_location};
use crate::link_validator::LinkCheckResult;
use crate::report::{Report, Severity};
use colored::{ColoredString, Colorize};
use std::env;

/// Decides depending on `--quiet` and `--format` what is printed.
///
/// With the json format nothing is printed while the links are checked, so stdout only contains
/// the final document.
#[derive(Clone, Copy)]
pub struct Printer {
    json: bool,
    /// Lines for links which are fine or skipped
    show_ok: bool,
    /// Lines for warnings and errors
    show_problems: bool,
    /// Print problems as GitHub workflow commands
    github_annotations: bool,
}

impl Printer {
    pub fn new(config: &OptionalConfig) -> Self {
        let json = config.is_json();
        Printer {
            json,
            show_ok: !json && !config.is_quiet(),
            show_problems: !json,
            github_annotations: !json && env::var("GITHUB_ENV").is_ok(),
        }
    }

    pub fn github_annotations(&self) -> bool {
        self.github_annotations
    }

    pub fn skipped(&self, link: &MarkupLink, reason: &str) {
        if self.show_ok {
            print_link(link, &"Skip".green(), reason, false);
        }
    }

    pub fn result(&self, link: &MarkupLink, result: &LinkCheckResult) {
        match result {
            LinkCheckResult::Ok if self.show_ok => print_link(link, &"OK".green(), "", false),
            LinkCheckResult::Ignored(msg) if self.show_ok => {
                print_link(link, &"Skip".green(), msg, false);
            }
            LinkCheckResult::NotImplemented(msg) | LinkCheckResult::Warning(msg)
                if self.show_problems =>
            {
                print_link(link, &"Warn".yellow(), msg, false);
            }
            LinkCheckResult::Failed(msg) if self.show_problems => {
                print_link(link, &"Err".red(), msg, true);
            }
            _ => {}
        }
    }

    /// Prints a warning or error as GitHub workflow command, see
    /// <https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands>
    pub fn annotation(&self, link: &MarkupLink, target: &str, result: &LinkCheckResult) {
        if !self.github_annotations {
            return;
        }
        let (level, title, msg) = match result {
            LinkCheckResult::NotImplemented(msg) | LinkCheckResult::Warning(msg) => {
                ("warning", "link checker warning", msg)
            }
            LinkCheckResult::Failed(msg) => ("error", "broken link", msg),
            LinkCheckResult::Ok | LinkCheckResult::Ignored(_) => return,
        };
        println!(
            "::{level} file={},line={},col={},title={title}::{target}. {msg}",
            link.source, link.line, link.column
        );
    }

    pub fn broken_reference(&self, broken_reference: &BrokenExtractedLink) {
        if self.show_problems {
            println!(
                "[{:^4}] {}:{}:{} => {} - {}",
                "Warn".yellow(),
                broken_reference.source,
                broken_reference.line,
                broken_reference.column,
                broken_reference.reference,
                broken_reference.error
            );
        }
    }

    /// Prints the final result: The json document or the summary and the list of broken links.
    pub fn report(&self, report: &Report) {
        if self.json {
            println!("{}", report.to_json());
            return;
        }

        let summary = &report.summary;
        println!();
        println!("Result ({} links):", summary.total);
        println!();
        println!("OK       {}", summary.ok);
        println!("Skipped  {}", summary.skipped);
        println!("Warnings {}", summary.warnings);
        println!("Errors   {}", summary.errors);
        println!();

        // In quiet mode the errors were just printed, so there is no need to repeat them
        if summary.errors > 0 && self.show_ok {
            println!();
            println!("The following links could not be resolved:");
            println!();
            for issue in report
                .issues
                .iter()
                .filter(|issue| issue.severity == Severity::Error)
            {
                println!(
                    "{}",
                    format_location(&issue.source, issue.line, issue.column, &issue.target)
                );
            }
        }
    }
}

fn print_link(link: &MarkupLink, status_code: &ColoredString, msg: &str, error_channel: bool) {
    let mut line = format!("[{:^4}] {}", status_code, link.source_str());
    if !msg.is_empty() {
        line += " - ";
        line += msg;
    }
    if error_channel {
        eprintln!("{line}");
    } else {
        println!("{line}");
    }
}
