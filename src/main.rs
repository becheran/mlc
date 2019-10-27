use linkchecker::cli;
use std::process;

fn main() {
    let config = cli::parse_args();
    if let Err(_) = linkchecker::run(&config) {
        process::exit(1);
    } else {
        process::exit(0);
    }
}
