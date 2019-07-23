use std::io::{BufReader, BufRead};
use std::fs::File;
use regex::Regex;
use crate::link::Link;

pub fn find_links(file: &str) -> Vec<Link> {
    let mut retval: Vec<Link> = Vec::new();
    info!("Scan file '{}' for links.", file);
    let buffered = BufReader::new(File::open(file).unwrap());

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
            let link = Link { line_nr, target, source: file.to_string() };
            retval.push(link);
        }
    }
    retval
}