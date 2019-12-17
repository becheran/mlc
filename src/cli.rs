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
    let markup_types = vec![MarkupType::Markdown]; //TODO read from cli
    debug!("The log level is: {}", log_level);
    debug!("The root folder is: {:?}", folder);
    debug!("The file extension are: {:?}", markup_types);
    Config {
        log_level,
        folder: folder.parse().unwrap(),
        markup_types,
    }
}
