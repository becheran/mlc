use super::html_link_extractor::HtmlLinkExtractor;
use super::markdown_link_extractor::MarkdownLinkExtractor;
use crate::markup::{MarkupFile, MarkupType};
use std::fmt;
use std::fs;

/// Link found in markup files
#[derive(PartialEq, Clone)]
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

impl fmt::Debug for MarkupLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} => {} (line {}, column {})",
            self.source, self.target, self.line, self.column
        )
    }
}

pub fn find_links(file: &MarkupFile) -> Vec<MarkupLink> {
    let path = &file.path;
    let link_extractor = link_extractor_factory(&file.markup_type);

    info!("Scan file at path '{}' for links.", path);
    match fs::read_to_string(path) {
        Ok(text) => {
            let mut links = link_extractor.find_links(&text);
            for l in &mut links {
                l.source = path.to_string();
            }
            links
        }
        Err(e) => {
            warn!(
                "File '{}'. IO Error: \"{}\". Check your file encoding.",
                path, e
            );
            vec![]
        }
    }
}

fn link_extractor_factory(markup_type: &MarkupType) -> Box<dyn LinkExtractor> {
    match markup_type {
        MarkupType::Markdown => Box::new(MarkdownLinkExtractor()),
        MarkupType::Html => Box::new(HtmlLinkExtractor()),
    }
}

pub trait LinkExtractor {
    fn find_links(&self, text: &str) -> Vec<MarkupLink>;
}
