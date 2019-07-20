use linkchecker::{file_traversal, logger};
use std::env;
use std::path::PathBuf;

#[test]
fn find_markdown_files() {
    logger::init(&logger::LogLevel::Debug);
    let file_extension = [".md"];
    let default_path = PathBuf::from(r"/");
    let mut path = env::current_dir().unwrap_or(default_path).to_string_lossy().to_string();
    path.push_str("/tests/traversal_test/three_empty_md_files");
    let mut result: Vec<String> = Vec::new();
    file_traversal::find(&path, &file_extension, &mut result);
    assert_eq!(result.len(), 3);
    assert!(result.contains(&"f1.md".to_string()));
    assert!(result.contains(&"f2.MD".to_string()));
    assert!(result.contains(&"F3_with_umlaut.md".to_string()));
}