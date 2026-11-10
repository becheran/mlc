use crate::markup::MarkupType;
use crate::report::OutputFormat;
use serde::Deserialize;
use std::path::PathBuf;
use wildmatch::WildMatch;

/// Everything which can be set via command line arguments or the `.mlc.toml` config file
#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OptionalConfig {
    pub debug: Option<bool>,
    pub do_not_warn_for_redirect_to: Option<Vec<String>>,
    pub markup_types: Option<Vec<MarkupType>>,
    pub offline: Option<bool>,
    pub match_file_extension: Option<bool>,
    pub ignore_links: Option<Vec<String>>,
    pub ignore_path: Option<Vec<PathBuf>>,
    pub root_dir: Option<PathBuf>,
    #[serde(rename = "csv")]
    pub csv_file: Option<PathBuf>,
    pub gitignore: Option<bool>,
    pub gituntracked: Option<bool>,
    pub throttle: Option<u32>,
    pub files: Option<Vec<PathBuf>>,
    pub http_headers: Option<Vec<String>>,
    pub quiet: Option<bool>,
    pub format: Option<OutputFormat>,
}

impl OptionalConfig {
    /// Only warnings, errors and the summary are printed.
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.quiet.unwrap_or(false)
    }

    /// The result is written as a single JSON document to stdout.
    #[must_use]
    pub fn is_json(&self) -> bool {
        self.format == Some(OutputFormat::Json)
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    pub directory: PathBuf,
    pub optional: OptionalConfig,
}

/// Compiles a list of `?` and `*` wildcard patterns
pub(crate) fn wildcards(patterns: Option<&[String]>) -> Vec<WildMatch> {
    patterns
        .unwrap_or_default()
        .iter()
        .map(|pattern| WildMatch::new(pattern))
        .collect()
}
