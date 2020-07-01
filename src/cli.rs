use crate::logger;
use crate::markup::MarkupType;
use crate::Config;
use clap::{App, Arg};
use wildmatch::WildMatch;
use std::path::Path;


pub fn parse_args() -> Config {
    let matches = App::new(crate_name!())
        .arg(
            Arg::with_name("directory")
                .help("Check all links in given directory and subdirectory")
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
        .arg(
            Arg::with_name("ignore_links")
                .long("ignore-links")
                .help("List of links which will not be checked")
                .min_values(1)
                .required(false),
        )
        .arg(
            Arg::with_name("markup_types")
                .long("markup-types")
                .short("t")
                .help("List of markup types which shall be checked")
                .min_values(1)
                .possible_values(&["md", "html"])
                .required(false),
        )        
        .arg(
            Arg::with_name("root_dir")
                .long("root-dir")
                .takes_value(true)
                .short("r")
                .help("Path to the root folder used to resolve all relative paths")
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
    let directory = matches
        .value_of("directory")
        .unwrap_or("./")
        .parse()
        .unwrap();

    let mut markup_types = vec![MarkupType::Markdown, MarkupType::HTML];
    if let Some(types) = matches.values_of("markup_types") {
        markup_types = types.map(|x| x.parse().unwrap()).collect();
    }

    let no_web_links = matches.is_present("no_web_links");

    let ignore_links: Vec<WildMatch> = matches
        .values_of("ignore_links")
        .unwrap_or_default()
        .map(|x| WildMatch::new(x))
        .collect();
    
    let root_dir = if let Some(root_path) = matches.value_of("root_dir"){
        let root_path = Path::new(root_path).to_path_buf();
        if !root_path.is_dir(){
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
        ignore_links,
        root_dir,
    }
}
