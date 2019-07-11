use std::{io, process};
use linkchecker::{Config};


fn main() {
    let config = Config::new();
    if let Err(e) = link-checker::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}