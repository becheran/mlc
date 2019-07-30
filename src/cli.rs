use crate::Config;
use crate::logger;
use clap::App;
use crate::markup::MarkupType;


pub fn parse_args() -> Config {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let debug = matches.is_present("debug");
    let log_level = if debug { logger::LogLevel::Debug } else { logger::LogLevel::Warn };
    let folder = matches.value_of("folder").unwrap_or("./");
    let markup_types = vec![MarkupType::Markdown]; //TODO read from cli
    debug!("The log level is: {}", log_level);
    debug!("The root folder is: {:?}", folder);
    debug!("The file extension are: {:?}", markup_types);
    Config { log_level, folder: folder.parse().unwrap(), markup_types }
}