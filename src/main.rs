use mlc::cli;
use std::process;

fn main() {
    let config = cli::parse_args();
    if let Err(_) = mlc::run(&config) {
        process::exit(1);
    } else {
        process::exit(0);
    }
}
