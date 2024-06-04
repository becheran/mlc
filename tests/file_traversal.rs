#[cfg(test)]
use mlc::cli::collect_ignore_paths;
use mlc::file_traversal;
use mlc::markup::{MarkupFile, MarkupType};
use mlc::Config;
use mlc::OptionalConfig;
use std::path::{Path, PathBuf};

#[test]
fn find_markdown_files() {
    let path = Path::new("./benches/benchmark/markdown/md_file_endings").to_path_buf();
    let config: Config = Config {
        directory: path,
        optional: OptionalConfig {
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    };
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config, &mut result);
    assert_eq!(result.len(), 12);
}

#[test]
fn empty_folder() {
    let path = Path::new("./benches/benchmark/markdown/empty").to_path_buf();
    let config: Config = Config {
        directory: path,
        optional: OptionalConfig {
        markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    };
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config, &mut result);
    assert!(result.is_empty());
}

#[test]
fn glob_test() {
    let options = glob::MatchOptions {
        case_sensitive: true,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let dir = PathBuf::from("./benches/benchmark");
    let c_dir = std::fs::canonicalize(dir).expect("Canonicalize failed");

    let glob_dir = "./benches/ben*".to_string();
    let ignore_paths = vec![&glob_dir];

    let collected_paths = collect_ignore_paths(ignore_paths.into_iter(), options);

    assert!(
        collected_paths.contains(&c_dir),
        "The expected globbed path is not in the collected paths: {:?}",
        collected_paths
    );
}
