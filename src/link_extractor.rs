use std::io::{BufReader, BufRead};
use std::fs::File;
use regex::Regex;
use crate::link::Link;
use crate::markup::{MarkupFile, MarkupType};

pub fn find_links(file: &MarkupFile) -> Vec<Link> {
    let path = &file.path;
    let link_extractor = link_extractor_factory(&file.markup_type);
    let mut retval: Vec<Link> = Vec::new();

    info!("Scan file at path '{}' for links.", path);
    let buffered = BufReader::new(File::open(path).unwrap());
    for (line_ctr, line_str) in buffered.lines().enumerate() {
        let line_str = line_str.unwrap();
        let line_nr = line_ctr + 1;
        let inline_links = &link_extractor.inline_links(&line_str);
        for inline_link in inline_links {
            let target = inline_link[inline_link.rfind('(').unwrap() + 1..(inline_link.len() - 1)].to_string();
            debug!("Found link '{}' in line {}", &target, line_nr);
            let link = Link { line_nr, target, source: path.clone() };
            retval.push(link);
        }
    }
    retval
}

struct MarkdownLinkExtractor();

trait LinkExtractor {
    fn inline_links<'a>(&self, text: &'a str) -> Vec<&'a str>;
    fn reference_links_source<'a>(&self, text: &'a str) -> Vec<&'a str>;
    fn reference_links_sink<'a>(&self, text: &'a str) -> Vec<&'a str>;
}

impl LinkExtractor for MarkdownLinkExtractor {
    fn inline_links<'a>(&self, text: &'a str) -> Vec<&'a str> {
        lazy_static! {
            static ref MARKDOWN_LINK_REGEX : Regex = Regex::new(
                    r"\[.*\]\(.*\)"
                ).unwrap();
        }
        MARKDOWN_LINK_REGEX.find_iter(&text).map(|mat| mat.as_str()).collect::<Vec<&str>>()
    }

    fn reference_links_source<'a>(&self, text: &'a str) -> Vec<&'a str> {
        unimplemented!()
    }

    fn reference_links_sink<'a>(&self, text: &'a str) -> Vec<&'a str> {
        unimplemented!()
    }
}

fn link_extractor_factory(markup_type: &MarkupType) -> impl LinkExtractor {
    match markup_type {
        MarkupType::Markdown => { MarkdownLinkExtractor() }
        MarkupType::HTML => { unimplemented!() }
    }
}