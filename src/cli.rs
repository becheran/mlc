use crate::logger;
use crate::markup::MarkupType;
use crate::Config;
use clap::Arg;
use clap::ArgAction;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::path::MAIN_SEPARATOR;
use wildmatch::WildMatch;

#[must_use]
pub fn parse_args() -> Config {    
    let matches = command!()
        .arg(Arg::new("directory")
            .help("Check all links in given directory and subdirectory")
            .required(false)
            .index(1))
        .arg(arg!(-d --debug "Print debug information to console")
            .required(false))
        .arg(arg!(-o --offline "Do not check web links")
            .alias("no-web-links")
            .required(false))
        .arg(Arg::new("match-file-extension")
            .long("match-file-extension")
            .short('e')
            .action(ArgAction::SetTrue)
            .help("Check the exact file extension when searching for a file")
            .required(false))
        .arg(Arg::new("ignore-path")
            .long("ignore-path")
            .short('p')
            .help("Comma separated list of files and directories which will be ignored")
            .value_name("PATHS")
            .value_delimiter(',')
            .action(ArgAction::Append)
            .required(false))
        .arg(Arg::new("ignore-links")
            .long("ignore-links")
            .short('i')
            .value_name("LINKS")
            .value_delimiter(',')
            .action(ArgAction::Append)
            .help("Comma separated list of links which will be ignored")
            .required(false))
        .arg(Arg::new("markup-types")
            .long("markup-types")
            .short('t')
            .value_name("TYPES")
            .help("Comma separated list of markup types which shall be checked")
            .action(ArgAction::Append)
            .value_delimiter(',')
            .required(false))
        .arg(arg!(-T --throttle <DELAY_MS> "Wait time in milliseconds between http request to the same host")
            .required(false))
        .arg(Arg::new("root-dir")
            .long("root-dir")
            .short('r')
            .value_name("DIR")
            .help("Path to the root folder used to resolve all relative paths")
            .required(false))
        .get_matches();

    let log_level = if matches.get_flag("debug") {
        logger::LogLevel::Debug
    } else {
        logger::LogLevel::Warn
    };

    let throttle: u32 = *matches.get_one::<u32>("throttle").unwrap_or(&0);

    let default_dir = format!(".{}",&MAIN_SEPARATOR);
    let dir_string = matches.get_one::<String>("directory").unwrap_or(&default_dir);
    let directory = dir_string
        .replace('/', &MAIN_SEPARATOR.to_string())
        .replace('\\', &MAIN_SEPARATOR.to_string())
        .parse()
        .expect("failed to parse path");

    let markup_types_str = matches
        .get_many::<String>("markup-types")
        .unwrap_or_default()
        .map(|v| v.as_str());
    let mut markup_types: Vec<MarkupType> = markup_types_str.map(|x| x.parse().expect("invalid markup type")).collect();
    if markup_types.is_empty(){
        markup_types = vec![MarkupType::Markdown, MarkupType::Html]
    }

    let no_web_links = matches.get_flag("offline");

    let match_file_extension = matches.get_flag("match-file-extension");

    let ignore_links: Vec<WildMatch> = matches
        .get_many::<String>("ignore-links")
        .unwrap_or_default()
        .map(|x|{
            WildMatch::new(x)
        })
        .collect();

    let ignore_path: Vec<PathBuf> = matches
        .get_many::<String>("ignore-path")
        .unwrap_or_default()
        .map(|x| {
            let path = Path::new(x).to_path_buf();
            match fs::canonicalize(&path) {
                Ok(p) => p,
                Err(e) => panic!("Ignore path {:?} not found. {:?}.", &path, e),
            }
        })
        .collect();

    let root_dir = if let Some(root_path) = matches.get_one::<String>("root-dir") {
        let root_path = Path::new(
            &root_path
                .replace('/', &MAIN_SEPARATOR.to_string())
                .replace('\\', &MAIN_SEPARATOR.to_string()),
        )
        .to_path_buf();
        if !root_path.is_dir() {
            eprintln!("Root path {:?} must be a directory!", root_path);
            std::process::exit(1);
        }
        Some(root_path)
    } else {
        None
    };

    Config {
        log_level,
        folder: directory,
        markup_types,
        no_web_links,
        match_file_extension,
        ignore_links,
        ignore_path,
        root_dir,
        throttle,
    }
}
