#[cfg(test)]
use mlc::file_traversal;
use mlc::markup::{MarkupFile, MarkupType};
use mlc::Config;
use mlc::OptionalConfig;
use std::path::Path;

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
