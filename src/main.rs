use std::process;
use linkchecker::cli;


fn main() {
    let config = cli::parse_args();
    if let Err(e) = linkchecker::run(&config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}