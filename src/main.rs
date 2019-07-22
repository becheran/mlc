use std::process;
use linkchecker::cli;


fn main() {
    let config = cli::parse_args();
    if let Err(e) = linkchecker::run(&config) {
        eprintln!("{}", e);
        process::exit(1);
    } else {
        println!("Process ended successfully.");
    }
}