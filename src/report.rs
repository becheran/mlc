use crate::link_extractors::link_extractor::{BrokenExtractedLink, MarkupLink};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;

/// How the result of a run is written to stdout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Human readable lines and a summary
    Text,
    /// A single JSON document, see [`Report`]
    Json,
}

impl FromStr for OutputFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<OutputFormat, ()> {
        match s {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err(()),
        }
    }
}

/// Errors are ordered before warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    fn csv_label(self) -> &'static str {
        match self {
            Severity::Error => "ERR",
            Severity::Warning => "WARN",
        }
    }
}

/// A broken link (error) or a suspicious link (warning)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub severity: Severity,
    /// The file the link was found in
    pub source: String,
    pub line: usize,
    pub column: usize,
    /// The link as written in the source file
    pub target: String,
    pub message: String,
}

impl Issue {
    pub(crate) fn from_link(severity: Severity, link: &MarkupLink, message: &str) -> Self {
        Issue {
            severity,
            source: link.source.clone(),
            line: link.line,
            column: link.column,
            target: link.target.clone(),
            message: message.to_string(),
        }
    }

    pub(crate) fn from_broken_reference(broken_reference: &BrokenExtractedLink) -> Self {
        Issue {
            severity: Severity::Warning,
            source: broken_reference.source.clone(),
            line: broken_reference.line,
            column: broken_reference.column,
            target: broken_reference.reference.clone(),
            message: broken_reference.error.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub total: usize,
    pub ok: usize,
    pub skipped: usize,
    pub warnings: usize,
    pub errors: usize,
}

/// Machine readable result of a run. Only problems are listed in `issues`,
/// links which are fine or skipped are only part of the `summary`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub summary: Summary,
    pub issues: Vec<Issue>,
}

impl Report {
    /// Creates a report with `total` filled in and the issues sorted (errors first, then by
    /// location), so the output is stable even though the links are checked concurrently.
    #[must_use]
    pub fn new(mut summary: Summary, mut issues: Vec<Issue>) -> Self {
        summary.total = summary.ok + summary.skipped + summary.warnings + summary.errors;
        issues.sort_by(|a, b| {
            (a.severity, &a.source, a.line, a.column, &a.target)
                .cmp(&(b.severity, &b.source, b.line, b.column, &b.target))
        });
        Report { summary, issues }
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("report is always serializable")
    }

    /// Writes all issues in the format `source,line,column,target,severity`
    pub fn write_csv(&self, path: &Path) -> io::Result<()> {
        let mut file = BufWriter::new(File::create(path)?);
        writeln!(file, "source,line,column,target,severity")?;
        for issue in &self.issues {
            writeln!(
                file,
                "{},{},{},{},{}",
                issue.source,
                issue.line,
                issue.column,
                issue.target,
                issue.severity.csv_label()
            )?;
        }
        file.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(severity: Severity, source: &str, line: usize) -> Issue {
        Issue {
            severity,
            source: source.to_string(),
            line,
            column: 1,
            target: "t".to_string(),
            message: "m".to_string(),
        }
    }

    #[test]
    fn output_format_from_str() {
        assert_eq!("text".parse(), Ok(OutputFormat::Text));
        assert_eq!("json".parse(), Ok(OutputFormat::Json));
        assert_eq!("xml".parse::<OutputFormat>(), Err(()));
    }

    #[test]
    fn issues_are_sorted_errors_first_then_by_location() {
        let report = Report::new(
            Summary::default(),
            vec![
                issue(Severity::Error, "b.md", 1),
                issue(Severity::Warning, "a.md", 7),
                issue(Severity::Error, "a.md", 2),
            ],
        );
        let issues: Vec<_> = report
            .issues
            .iter()
            .map(|i| (i.severity, i.source.as_str(), i.line))
            .collect();
        assert_eq!(
            issues,
            vec![
                (Severity::Error, "a.md", 2),
                (Severity::Error, "b.md", 1),
                (Severity::Warning, "a.md", 7),
            ]
        );
    }

    #[test]
    fn total_is_the_sum_of_all_links() {
        let summary = Summary {
            ok: 4,
            skipped: 3,
            warnings: 2,
            errors: 1,
            ..Default::default()
        };
        assert_eq!(Report::new(summary, vec![]).summary.total, 10);
    }

    #[test]
    fn csv_has_header_and_one_line_per_issue() {
        let path = std::env::temp_dir().join(format!("mlc_report_{}.csv", std::process::id()));
        let report = Report::new(
            Summary::default(),
            vec![
                issue(Severity::Warning, "a.md", 7),
                issue(Severity::Error, "b.md", 1),
            ],
        );
        report.write_csv(&path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            content.lines().collect::<Vec<_>>(),
            vec![
                "source,line,column,target,severity",
                "b.md,1,1,t,ERR",
                "a.md,7,1,t,WARN",
            ]
        );
    }

    #[test]
    fn json_has_documented_shape() {
        let summary = Summary {
            ok: 1,
            errors: 1,
            ..Default::default()
        };
        let report = Report::new(summary, vec![issue(Severity::Error, "a.md", 3)]);
        let json: serde_json::Value = serde_json::from_str(&report.to_json()).unwrap();
        assert_eq!(json["summary"]["total"], 2);
        assert_eq!(json["summary"]["errors"], 1);
        assert_eq!(json["issues"][0]["severity"], "error");
        assert_eq!(json["issues"][0]["source"], "a.md");
        assert_eq!(json["issues"][0]["line"], 3);
        assert_eq!(json["issues"][0]["column"], 1);
        assert_eq!(json["issues"][0]["target"], "t");
        assert_eq!(json["issues"][0]["message"], "m");
    }
}
