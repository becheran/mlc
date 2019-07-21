use std::collections::HashSet;

use std::io::{BufReader, BufRead};
use std::fs::File;
use regex::Regex;

pub fn find_links<'a>(file: &str) -> HashSet<&str> {
    let mut ret_set: HashSet<&str> = HashSet::new();
    let buffered = BufReader::new(File::open(file).unwrap());

    lazy_static! {
        static ref HASHTAG_REGEX : Regex = Regex::new(
                r"\#[a-zA-Z][0-9a-zA-Z_]*"
            ).unwrap();
    }

    for line in buffered.lines() {
        let line_str = line.unwrap();
        let links = HASHTAG_REGEX.find_iter(&line_str).map(|mat| mat.as_str()).collect::<HashSet<&str>>();
        for link in links{
            ret_set.insert(link);
        }
    }
    ret_set
}