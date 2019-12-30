use crate::logger;
use crate::markup::MarkupType;
use crate::Config;
use clap::{App, Arg};

pub fn parse_args() -> Config {
    let matches = App::new(crate_name!())
        .arg(
            Arg::with_name("folder")
                .help("Check all links in given folder and subfolders")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .short("d")
                .help("Print debug information to console")
                .required(false),
        )
        .arg(
            Arg::with_name("no_web_links")
                .long("no-web-links")
                .help("Do not check web links")
                .required(false),
        )
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .get_matches();
    let debug = matches.is_present("debug");
    let log_level = if debug {
        logger::LogLevel::Debug
    } else {
        logger::LogLevel::Warn
    };
    let folder = matches.value_of("folder").unwrap_or("./");
    let markup_types = vec![MarkupType::Markdown];
    let no_web_links = matches.is_present("no_web_links");
    Config {
        log_level,
        folder: folder.parse().unwrap(),
        markup_types,
        no_web_links,
    }
}
