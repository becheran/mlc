use super::html_link_extractor::HtmlLinkExtractor;
use super::markdown_link_extractor::MarkdownLinkExtractor;
use crate::markup::{MarkupFile, MarkupType};
use std::env;
use std::fmt;
use std::fs;

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
    pub fn source_str(&self) -> String {
        lazy_static! {
            static ref IS_VS_CODE_TERMINAL: bool =
                env::var("TERM_PROGRAM") == Ok("vscode".to_string());
        }
        if *IS_VS_CODE_TERMINAL {
            format! {"{}:{}:{} => {}", self.source, self.line, self.column, self.target}
        } else {
            format! {"{} ({}, {}) => {}", self.source, self.line, self.column, self.target}
        }
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
            for l in &mut links {
                match l {
                    Ok(link) => {
                        link.source = path.to_string();
                    }
                    Err(broken_link) => {
                        broken_link.source = path.to_string();
                    }
                }
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
