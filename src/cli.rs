use crate::Config;
use crate::logger;
use clap::App;


pub fn parse_args() -> Config {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let debug = matches.is_present("debug");
    let log_level = if debug { logger::LogLevel::Debug } else { logger::LogLevel::Warn };
    let folder = matches.value_of("folder").unwrap_or("./");
    let file_extensions = vec![".md".to_string()]; //TODO add other options
    debug!("The log level is: {}", log_level);
    debug!("The root folder is: {:?}", file_extensions);
    debug!("The file extension are: {:?}", file_extensions);
    Config { log_level, folder: folder.parse().unwrap(), file_extensions: file_extensions }
}