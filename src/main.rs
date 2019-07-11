use std::{process};
use linkchecker::Config;


fn main() {
    let config = Config::new();
    if let Err(e) = linkchecker::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}