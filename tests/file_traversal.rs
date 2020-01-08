#[cfg(test)]
use mlc::file_traversal;
use mlc::markup::{MarkupFile, MarkupType};
use mlc::Config;

#[test]
fn find_markdown_files() {
    let path = "./benches/benchmark/markdown/md_file_endings".to_string();
    let config: Config = Config {
        folder: path,
        markup_types: vec![MarkupType::Markdown],
        ..Default::default()
    };
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config, &mut result);
    assert_eq!(result.len(), 12);
}

#[test]
fn empty_folder() {
    let path = "./benches/benchmark/markdown/empty".to_string();
    let config: Config = Config {
        folder: path,
        markup_types: vec![MarkupType::Markdown],
        ..Default::default()
    };
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config, &mut result);
    assert!(result.is_empty());
}
