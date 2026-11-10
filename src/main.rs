use clap::crate_version;
use log::info;
use mlc::{Error, cli, logger};
use std::process::ExitCode;

fn print_header() {
    const WIDTH: usize = 60;
    let inner_width = WIDTH - 2;
    let header = format!("markup link checker - mlc v{}", crate_version!());
    let border = "+".repeat(WIDTH);
    let empty_line = format!("+{:inner_width$}+", "");
    println!("\n{border}\n{empty_line}\n+{header:^inner_width$}+\n{empty_line}\n{border}\n");
}

#[tokio::main]
async fn main() -> ExitCode {
    let config = cli::parse_args();
    if !config.optional.is_quiet() && !config.optional.is_json() {
        print_header();
    }
    let log_level = if config.optional.debug == Some(true) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Error
    };
    // Keep stdout reserved for the JSON document
    if let Err(e) = logger::init(log_level, config.optional.is_json()) {
        eprintln!("Failed to initialize logging: {e}");
        return ExitCode::FAILURE;
    }
    info!("Config: {config:#?}");

    match mlc::run(&config).await {
        Ok(()) => ExitCode::SUCCESS,
        // The broken links were already reported
        Err(Error::BrokenLinks) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
