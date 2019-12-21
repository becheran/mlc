#[cfg(test)]
use mlc::file_traversal;
use mlc::markup::{MarkupFile, MarkupType};
use mlc::Config;
use std::path::Path;

#[test]
fn find_markdown_files() {
    let path = "./benches/benchmark/three_empty_md_files".to_string();
    let config: Config = Config {
        folder: path,
        markup_types: vec![MarkupType::Markdown],
        ..Default::default()
    };
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config, &mut result);
    assert_eq!(result.len(), 3);
    let possible_results = ["f1.md", "f2.MD", "F3_with_umlaut.md"];
    for r in result {
        let path = r.path;
        let file_names = &Path::new(&path).file_name().unwrap().to_str().unwrap();
        assert!(possible_results.contains(file_names));
    }
}

#[test]
fn empty_folder() {
    let path = "./benches/benchmark/empty".to_string();
    let config: Config = Config {
        folder: path,
        markup_types: vec![MarkupType::Markdown],
        ..Default::default()
    };
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config, &mut result);
    assert!(result.is_empty());
}
