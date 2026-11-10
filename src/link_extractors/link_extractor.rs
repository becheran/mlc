use super::html_link_extractor::HtmlLinkExtractor;
use super::markdown_link_extractor::MarkdownLinkExtractor;
use crate::markup::{MarkupFile, MarkupType};
use log::{info, warn};
use std::env;
use std::fmt;
use std::fs;
use std::sync::LazyLock;

/// VS Code only makes `file:line:column` clickable in its terminal
static IS_VS_CODE_TERMINAL: LazyLock<bool> =
    LazyLock::new(|| env::var("TERM_PROGRAM").is_ok_and(|program| program == "vscode"));

/// Human readable position of a link, for example `README.md (3, 1) => ./missing.md`
#[must_use]
pub fn format_location(source: &str, line: usize, column: usize, target: &str) -> String {
    if *IS_VS_CODE_TERMINAL {
        format!("{source}:{line}:{column} => {target}")
    } else {
        format!("{source} ({line}, {column}) => {target}")
    }
}

/// Link found in markup files
#[derive(Eq, PartialEq, Clone)]
pub struct MarkupLink {
    /// The source file of the link
    pub source: String,
    /// The target the link points to
    pub target: String,
    /// The line number were the link was found
    pub line: usize,
    /// The column number were the link was found
    pub column: usize,
}
/// Broken link found in document
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct BrokenExtractedLink {
    /// The error message
    pub error: String,
    /// The source
    pub source: String,
    /// The target
    pub reference: String,
    /// The line number were the link was found
    pub line: usize,
    /// The column number were the link was found
    pub column: usize,
}

impl fmt::Debug for MarkupLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} => {} (line {}, column {})",
            self.source, self.target, self.line, self.column
        )
    }
}

impl MarkupLink {
    #[must_use]
    pub fn source_str(&self) -> String {
        format_location(&self.source, self.line, self.column, &self.target)
    }
}

#[must_use]
pub fn find_links(file: &MarkupFile) -> Vec<Result<MarkupLink, BrokenExtractedLink>> {
    let path = &file.path;
    let link_extractor = link_extractor_factory(file.markup_type);

    info!("Scan file at path '{path}' for links.");
    match fs::read_to_string(path) {
        Ok(text) => {
            let mut links = link_extractor.find_links(&text);
            for link in &mut links {
                let source = match link {
                    Ok(link) => &mut link.source,
                    Err(broken_link) => &mut broken_link.source,
                };
                source.clone_from(path);
            }
            links
        }
        Err(e) => {
            warn!("File '{path}'. IO Error: \"{e}\". Check your file encoding.");
            vec![]
        }
    }
}

fn link_extractor_factory(markup_type: MarkupType) -> Box<dyn LinkExtractor> {
    match markup_type {
        MarkupType::Markdown => Box::new(MarkdownLinkExtractor()),
        MarkupType::Html => Box::new(HtmlLinkExtractor()),
    }
}

pub trait LinkExtractor {
    fn find_links(&self, text: &str) -> Vec<Result<MarkupLink, BrokenExtractedLink>>;
}
