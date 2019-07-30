use std::io::{BufReader, BufRead};
use std::fs::File;
use regex::Regex;
use crate::link::Link;
use crate::markup::MarkupFile;

pub fn find_links(file: &MarkupFile) -> Vec<Link> {
    let path = &file.path;
    let mut retval: Vec<Link> = Vec::new();
    info!("Scan file at path '{}' for links.", path);
    let buffered = BufReader::new(File::open(path).unwrap());

    lazy_static! {
        static ref MARKDOWN_LINK_REGEX : Regex = Regex::new(
                r"\[.*\]\(.*\)"
            ).unwrap();
    }

    for (line_ctr, line_str) in buffered.lines().enumerate() {
        let line_str = line_str.unwrap();
        let line_nr = line_ctr + 1;
        let md_links = MARKDOWN_LINK_REGEX.find_iter(&line_str).map(|mat| mat.as_str()).collect::<Vec<&str>>();
        for md_link in md_links {
            let target = md_link[md_link.rfind('(').unwrap() + 1..(md_link.len() - 1)].to_string();
            debug!("Found link '{}' in line {}", &target, line_nr);
            let link = Link { line_nr, target, source: path.clone() };
            retval.push(link);
        }
    }
    retval
}