use crate::Config;
use crate::OptionalConfig;
use crate::markup::MarkupType;
use clap::{Arg, ArgAction, ArgMatches, arg, command, value_parser};
use std::fs;
use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR, PathBuf};

const CONFIG_FILE_PATH: &str = "./.mlc.toml";

/// Exit code if the arguments or the config file are invalid
const EXIT_CODE_INVALID_INPUT: i32 = 2;

fn normalize_path_separators(path: &str) -> String {
    path.replace(['/', '\\'], MAIN_SEPARATOR_STR)
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(EXIT_CODE_INVALID_INPUT)
}

/// Like [`fs::canonicalize`], but only warns if the path does not exist
fn canonicalize_or_warn(description: &str, path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).unwrap_or_else(|e| {
        eprintln!("⚠ Warn: {description} {path:?} not found. {e:?}.");
        path
    })
}

fn read_config_file() -> OptionalConfig {
    match fs::read_to_string(CONFIG_FILE_PATH) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
            exit_with_error(&format!("Invalid TOML file {CONFIG_FILE_PATH}\n{e}"))
        }),
        Err(_) => OptionalConfig::default(),
    }
}

fn list(matches: &ArgMatches, id: &str) -> Option<Vec<String>> {
    matches.get_many::<String>(id).map(|v| v.cloned().collect())
}

fn command_line() -> clap::Command {
    // Comma separated lists can be given as `--arg a,b` or as `--arg a --arg b`
    let list_arg =
        |id: &'static str, short: Option<char>, value_name: &'static str, help: &'static str| {
            Arg::new(id)
                .long(id)
                .short(short)
                .value_name(value_name)
                .help(help)
                .value_delimiter(',')
                .action(ArgAction::Append)
                .required(false)
        };

    command!()
        .arg(
            Arg::new("directory")
                .help("Check all links in given directory and subdirectory")
                .required(false)
                .index(1),
        )
        .arg(arg!(-d --debug "Print debug information to console").required(false))
        .arg(
            arg!(-q --quiet "Only print warnings, errors and the summary. Skips the banner and all links which are ok or skipped")
                .required(false),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_name("FORMAT")
                .value_parser(["text", "json"])
                .help("Output format on stdout. With 'json' a single JSON document with the summary and all issues is printed and nothing else")
                .required(false),
        )
        .arg(
            arg!(-o --offline "Do not check web links")
                .alias("no-web-links")
                .required(false),
        )
        .arg(list_arg(
            "do-not-warn-for-redirect-to",
            None,
            "LINKS",
            "Comma separated list of links which will be ignored",
        ))
        .arg(
            Arg::new("match-file-extension")
                .long("match-file-extension")
                .short('e')
                .action(ArgAction::SetTrue)
                .help("Check the exact file extension when searching for a file")
                .required(false),
        )
        .arg(list_arg(
            "ignore-path",
            Some('p'),
            "PATHS",
            "Comma separated list of files and directories which will be ignored",
        ))
        .arg(list_arg(
            "ignore-links",
            Some('i'),
            "LINKS",
            "Comma separated list of links which will be ignored",
        ))
        .arg(
            list_arg(
                "markup-types",
                Some('t'),
                "TYPES",
                "Comma separated list of markup types which shall be checked",
            )
            .value_parser(["md", "html"]),
        )
        .arg(
            Arg::new("throttle")
                .long("throttle")
                .short('T')
                .value_name("DELAY-MS")
                .help("Wait time in milliseconds between http request to the same host")
                .value_parser(value_parser!(u32))
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
        .arg(list_arg(
            "files",
            Some('f'),
            "FILES",
            "Comma separated list of files which shall be checked",
        ))
        .arg(list_arg(
            "http-headers",
            Some('H'),
            "HEADERS",
            "Comma separated list of custom HTTP headers in the format 'Name: Value'. For example 'User-Agent: Mozilla/5.0'",
        ))
}

/// Reads the `.mlc.toml` config file and the command line arguments. The arguments win.
#[must_use]
pub fn parse_args() -> Config {
    let mut opt = read_config_file();
    let matches = command_line().get_matches();

    let default_dir = format!(".{MAIN_SEPARATOR}");
    let directory = matches
        .get_one::<String>("directory")
        .map_or(default_dir, |dir| normalize_path_separators(dir))
        .into();

    let flag = |id: &str| matches.get_flag(id).then_some(true);
    opt.debug = flag("debug").or(opt.debug);
    opt.quiet = flag("quiet").or(opt.quiet);
    opt.offline = flag("offline").or(opt.offline);
    opt.match_file_extension = flag("match-file-extension").or(opt.match_file_extension);
    opt.gitignore = flag("gitignore").or(opt.gitignore);
    opt.gituntracked = flag("gituntracked").or(opt.gituntracked);

    if let Some(format) = matches.get_one::<String>("format") {
        opt.format = format.parse().ok();
    }
    if let Some(throttle) = matches.get_one::<u32>("throttle") {
        opt.throttle = Some(*throttle);
    }
    if let Some(csv_file) = matches.get_one::<String>("csv") {
        opt.csv_file = Some(normalize_path_separators(csv_file).into());
    }

    opt.do_not_warn_for_redirect_to =
        list(&matches, "do-not-warn-for-redirect-to").or(opt.do_not_warn_for_redirect_to);
    opt.ignore_links = list(&matches, "ignore-links").or(opt.ignore_links);
    opt.http_headers = list(&matches, "http-headers").or(opt.http_headers);

    opt.markup_types = list(&matches, "markup-types")
        .map(|types| types.iter().filter_map(|t| t.parse().ok()).collect())
        .or(opt.markup_types)
        .or_else(|| Some(vec![MarkupType::Markdown, MarkupType::Html]));

    if let Some(ignore_path) = list(&matches, "ignore-path") {
        opt.ignore_path = Some(
            ignore_path
                .into_iter()
                .map(|path| canonicalize_or_warn("Ignore path", path.into()))
                .collect(),
        );
    }
    if let Some(files) = list(&matches, "files") {
        opt.files = Some(
            files
                .iter()
                .map(|file| {
                    canonicalize_or_warn("File path", normalize_path_separators(file).into())
                })
                .collect(),
        );
    }

    if let Some(root_dir) = matches.get_one::<String>("root-dir") {
        let root_path = PathBuf::from(normalize_path_separators(root_dir));
        if !root_path.is_dir() {
            exit_with_error(&format!("Root path {root_path:?} must be a directory!"));
        }
        opt.root_dir = Some(root_path);
    } else if let Some(root_path) = &opt.root_dir
        && !root_path.is_dir()
    {
        exit_with_error(&format!(
            "Root path {root_path:?} must be an existing directory (from {CONFIG_FILE_PATH})."
        ));
    }

    Config {
        directory,
        optional: opt,
    }
}
