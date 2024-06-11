#[macro_use]
extern crate log;

use mlc::cli;
use mlc::logger;
use std::process;

#[macro_use]
extern crate clap;

fn print_header() {
    let width = 60;
    let header = format!("markup link checker - mlc v{:}", crate_version!());
    println!();
    println!("{:+<1$}", "", width);
    print!("+");
    print!("{: <1$}", "", width - 2);
    println!("+");
    print!("+");
    print!("{: ^1$}", header, width - 2);
    println!("+");
    print!("+");
    print!("{: <1$}", "", width - 2);
    println!("+");
    println!("{:+<1$}", "", width);
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();
    let config = cli::parse_args();
    let log_level = match config.optional.debug {
        Some(true) => logger::LogLevel::Debug,
        _ => logger::LogLevel::Warn,
    };
    logger::init(&log_level);
    info!("Config: {}", &config);
    if mlc::run(&config).await.is_err() {
        process::exit(1);
    } else {
        process::exit(0);
    }
}
