use crate::markup::MarkupType;
use crate::Config;
use crate::OptionalConfig;
use clap::Arg;
use clap::ArgAction;
use std::fs;
use std::path::Path;
use std::path::MAIN_SEPARATOR;

const CONFIG_FILE_PATH: &str = "./.mlc.toml";

fn normalize_path_separators(path: &str) -> String {
    path.replace(['/', '\\'], std::path::MAIN_SEPARATOR_STR)
}

#[must_use]
pub fn parse_args() -> Config {
    let mut opt: OptionalConfig = match fs::read_to_string(CONFIG_FILE_PATH) {
        Ok(content) => match toml::from_str(&content) {
            Ok(o) => o,
            Err(err) => panic!("Invalid TOML file {:?}", err),
        },
        Err(_) => OptionalConfig::default(),
    };

    let matches = command!()
        .arg(
            Arg::new("directory")
                .help("Check all links in given directory and subdirectory")
                .required(false)
                .index(1),
        )
        .arg(arg!(-d --debug "Print debug information to console").required(false))
        .arg(
            arg!(-o --offline "Do not check web links")
                .alias("no-web-links")
                .required(false),
        )
        .arg(
            Arg::new("do-not-warn-for-redirect-to")
                .long("do-not-warn-for-redirect-to")
                .value_name("LINKS")
                .value_delimiter(',')
                .action(ArgAction::Append)
                .help("Comma separated list of links which will be ignored")
                .required(false),
        )
        .arg(
            Arg::new("match-file-extension")
                .long("match-file-extension")
                .short('e')
                .action(ArgAction::SetTrue)
                .help("Check the exact file extension when searching for a file")
                .required(false),
        )
        .arg(
            Arg::new("ignore-path")
                .long("ignore-path")
                .short('p')
                .help("Comma separated list of files and directories which will be ignored")
                .value_name("PATHS")
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(false),
        )
        .arg(
            Arg::new("ignore-links")
                .long("ignore-links")
                .short('i')
                .value_name("LINKS")
                .value_delimiter(',')
                .action(ArgAction::Append)
                .help("Comma separated list of links which will be ignored")
                .required(false),
        )
        .arg(
            Arg::new("markup-types")
                .long("markup-types")
                .short('t')
                .value_name("TYPES")
                .help("Comma separated list of markup types which shall be checked")
                .action(ArgAction::Append)
                .value_delimiter(',')
                .required(false),
        )
        .arg(
            Arg::new("throttle")
                .long("throttle")
                .short('T')
                .value_name("DELAY-MS")
                .help("Wait time in milliseconds between http request to the same host")
                .action(ArgAction::Append)
                .required(false),
        )
        .arg(
            Arg::new("root-dir")
                .long("root-dir")
                .short('r')
                .value_name("DIR")
                .help("Path to the root folder used to resolve all relative paths")
                .required(false),
        )
        .arg(
            Arg::new("gitignore")
                .long("gitignore")
                .short('g')
                .value_name("GIT")
                .help("Ignore all files ignored by git")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("csv")
                .long("csv")
                .value_name("CSV_FILE")
                .help("set the output file for the CSV report")
                .required(false),
        )
        .arg(
            Arg::new("gituntracked")
                .long("gituntracked")
                .short('u')
                .value_name("GITUNTRACKED")
                .help("Ignore all files untracked by git")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("disable-raw-link-check")
                .long("disable-raw-link-check")
                .short('c')
                .action(ArgAction::SetTrue)
                .help("Disable checking of raw links in code blocks and other text. By default, raw HTTP(S) URLs are extracted and checked.")
                .required(false),
        )
        .arg(
            Arg::new("files")
                .long("files")
                .short('f')
                .help("Comma separated list of files which shall be checked")
                .value_name("FILES")
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(false),
        )
        .arg(
            Arg::new("http-headers")
                .long("http-headers")
                .short('H')
                .help("Comma separated list of custom HTTP headers in the format 'Name: Value'. For example 'User-Agent: Mozilla/5.0'")
                .value_name("HEADERS")
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(false),
        )
        .get_matches();

    let default_dir = format!(".{}", &MAIN_SEPARATOR);
    let dir_string = matches
        .get_one::<String>("directory")
        .unwrap_or(&default_dir);
    let directory = normalize_path_separators(dir_string)
        .parse()
        .expect("failed to parse path");

    if matches.get_flag("debug") {
        opt.debug = Some(true);
    }

    if let Some(do_not_warn_for_redirect_to) =
        matches.get_many::<String>("do-not-warn-for-redirect-to")
    {
        opt.do_not_warn_for_redirect_to =
            Some(do_not_warn_for_redirect_to.map(|x| x.to_string()).collect());
    }

    if let Some(throttle_str) = matches.get_one::<String>("throttle") {
        let throttle = throttle_str.parse::<u32>().unwrap();
        opt.throttle = Some(throttle);
    }

    if let Some(f) = matches.get_one::<String>("csv") {
        opt.csv_file = Some(Path::new(&normalize_path_separators(f)).to_path_buf());
    }

    if let Some(markup_types) = matches.get_many::<String>("markup-types") {
        opt.markup_types = Some(
            markup_types
                .map(|v| v.as_str().parse().expect("invalid markup type"))
                .collect(),
        );
    }
    if opt.markup_types.is_none() {
        opt.markup_types = Some(vec![MarkupType::Markdown, MarkupType::Html]);
    }

    if matches.get_flag("offline") {
        opt.offline = Some(true);
    }

    if matches.get_flag("match-file-extension") {
        opt.match_file_extension = Some(true)
    }

    if let Some(ignore_links) = matches.get_many::<String>("ignore-links") {
        opt.ignore_links = Some(ignore_links.map(|x| x.to_string()).collect());
    }

    if let Some(ignore_path) = matches.get_many::<String>("ignore-path") {
        let mut paths: Vec<_> = ignore_path.map(|x| Path::new(x).to_path_buf()).collect();
        for p in paths.iter_mut() {
            match fs::canonicalize(&p) {
                Ok(canonical_path) => {
                    *p = canonical_path;
                }
                Err(e) => {
                    println!("⚠ Warn: Ignore path {p:?} not found. {e:?}.");
                }
            };
        }
        opt.ignore_path = Some(paths);
    }

    if matches.get_flag("gitignore") {
        opt.gitignore = Some(true);
    }

    if matches.get_flag("gituntracked") {
        opt.gituntracked = Some(true);
    }

    if matches.get_flag("disable-raw-link-check") {
        opt.disable_raw_link_check = Some(true);
    }

    if let Some(files) = matches.get_many::<String>("files") {
        let mut file_paths: Vec<_> = files
            .map(|x| Path::new(&normalize_path_separators(x)).to_path_buf())
            .collect();
        for p in file_paths.iter_mut() {
            match fs::canonicalize(&p) {
                Ok(canonical_path) => {
                    *p = canonical_path;
                }
                Err(e) => {
                    println!("⚠ Warn: File path {p:?} not found. {e:?}.");
                }
            };
        }
        opt.files = Some(file_paths);
    }

    if let Some(http_headers) = matches.get_many::<String>("http-headers") {
        opt.http_headers = Some(http_headers.map(|x| x.to_string()).collect());
    }

    if let Some(root_dir) = matches.get_one::<String>("root-dir") {
        let root_path = Path::new(&normalize_path_separators(root_dir)).to_path_buf();
        if !root_path.is_dir() {
            eprintln!("Root path {root_path:?} must be a directory!");
            std::process::exit(1);
        }
        opt.root_dir = Some(root_path)
    }

    Config {
        directory,
        optional: opt,
    }
}
