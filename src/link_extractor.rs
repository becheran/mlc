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
            debug!("Found inline link '{}' in line {}", inline_link, line_nr);
            let link = Link { line_nr, target: inline_link.to_string(), source: path.clone() };
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
        let mut result: Vec<&'a str> = Vec::new();
        lazy_static! {
            static ref INLINE_REGEX : Regex = Regex::new(
                    r"\[.*.*\]\(.*\)"
                ).unwrap();
        }
        lazy_static! {
            static ref SHORT_REGEX : Regex = Regex::new(
                    r"<..+:\S*>"
                ).unwrap();
        }

        let short_links: Vec<&str> = SHORT_REGEX.find_iter(&text)
            .map(|mat| mat.as_str()).collect::<Vec<&str>>();
        for sl in short_links {
            if sl.len() > 2 {
                let s = &sl[1..sl.len() - 1];
                result.push(s);
            }
        }

        let markdown_links: Vec<&str> = INLINE_REGEX.find_iter(&text)
            .map(|mat| mat.as_str()).collect::<Vec<&str>>();
        for md_links in markdown_links {
            let start_idx = md_links.rfind('(').unwrap() + 1;
            let end_idx = md_links.len() - 1;
            let link_with_title = &md_links[start_idx..end_idx];
            let mut spl = link_with_title.split_whitespace();
            let link = spl.next().unwrap();
            result.push(link);
        }
        result
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md_inline_no_link() {
        let le = MarkdownLinkExtractor();
        let input = "]This is not a () link](! has no title attribute.";
        let result = le.inline_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn md_inline_link_no_title() {
        let le = MarkdownLinkExtractor();
        let link = "http://example.net/";
        let input = format!("[This link]({}) has no title attribute.", link);
        let result = le.inline_links(&input);
        assert_eq!(vec![link], result);
    }

    #[test]
    fn md_inline_link_with_title() {
        let le = MarkdownLinkExtractor();
        let link = "http://example.net/";
        let input = format!("[This is a link]({} \"with title\") oh yea.", link);
        let result = le.inline_links(&input);
        assert_eq!(vec![link], result);
    }

    #[test]
    fn md_inline_link() {
        let le = MarkdownLinkExtractor();
        let link = "http://example.net/";
        let input = format!("This is a short link <{}>.", link);
        let result = le.inline_links(&input);
        assert_eq!(vec![link], result);
    }
}
