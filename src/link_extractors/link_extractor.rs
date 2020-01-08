use super::markdown_link_extractor::MarkdownLinkExtractor;
use super::html_link_extractor::HtmlLinkExtractor;
use crate::markup::{MarkupFile, MarkupType};
use std::fs;
use std::fmt;

/// Links found in markup files
#[derive(PartialEq)]
pub struct MarkupLink {
    /// The target the links points to
    pub target: String,
    /// The line number were the link was found
    pub line: usize,
    /// The column number were the link was found
    pub column: usize,
}

impl fmt::Debug for MarkupLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (line {}, column {})", self.target, self.line, self.column)
    }
}

pub fn find_links(file: &MarkupFile) -> Vec<MarkupLink> {
    let path = &file.path;
    let link_extractor = link_extractor_factory(&file.markup_type);

    info!("Scan file at path '{}' for links.", path);
    let text = fs::read_to_string(path).expect("File could not be opened.");
    link_extractor.find_links(&text)
}

fn link_extractor_factory(markup_type: &MarkupType) -> impl LinkExtractor {
    match markup_type {
        MarkupType::Markdown => MarkdownLinkExtractor(),
        MarkupType::HTML => unimplemented!(),
    }
}

pub trait LinkExtractor {
    fn find_links(&self, text: &str) -> Vec<MarkupLink>;
}
