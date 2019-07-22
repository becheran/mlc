use std::io::{BufReader, BufRead};
use std::fs::File;
use regex::Regex;

pub fn find_links<'a>(file: &str) -> Vec<String> {
    info!("Scan file '{}' for links.", file);
    let mut retval: Vec<String> = Vec::new();
    let buffered = BufReader::new(File::open(file).unwrap());

    lazy_static! {
        static ref MARKDOWN_LINK_REGEX : Regex = Regex::new(
                r"\[.*\]\(.*\)"
            ).unwrap();
    }

    for line in buffered.lines() {
        let line_str = line.unwrap();
        let md_links = MARKDOWN_LINK_REGEX.find_iter(&line_str).map(|mat| mat.as_str()).collect::<Vec<&str>>();
        for md_link in md_links {
            let link = md_link[md_link.rfind("(").unwrap() + 1..(md_link.len() - 1)].to_string();
            debug!("Found link '{}'", link);
            retval.push(link);
        }
    }
    retval
}