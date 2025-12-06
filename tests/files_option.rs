use mlc::file_traversal;
use mlc::markup::{MarkupFile, MarkupType};
use mlc::Config;
use mlc::OptionalConfig;
use std::path::{Path, PathBuf};

#[test]
fn find_specific_files() {
    let file1 = Path::new("./README.md").to_path_buf();
    let file2 = Path::new("./CHANGELOG.md").to_path_buf();
    
    let config: Config = Config {
        directory: PathBuf::from("."),
        optional: OptionalConfig {
            markup_types: Some(vec![MarkupType::Markdown]),
            files: Some(vec![file1, file2]),
            ..Default::default()
        },
    };
    
    let mut result: Vec<MarkupFile> = Vec::new();
    file_traversal::find(&config, &mut result);
    
    assert_eq!(result.len(), 2);
    assert!(result.iter().any(|f| f.path.contains("README.md")));
    assert!(result.iter().any(|f| f.path.contains("CHANGELOG.md")));
}

#[test]
fn find_single_file() {
    let file1 = Path::new("./README.md").to_path_buf();
    
    let config: Config = Config {
        directory: PathBuf::from("."),
        optional: OptionalConfig {
            markup_types: Some(vec![MarkupType::Markdown]),
            files: Some(vec![file1]),
            ..Default::default()
        },
    };
    
    let mut result: Vec<MarkupFile> = Vec::new();
    file_traversal::find(&config, &mut result);
    
    assert_eq!(result.len(), 1);
    assert!(result[0].path.contains("README.md"));
}

#[test]
fn find_files_ignores_non_matching_types() {
    // Test with a markdown file but only HTML markup type configured
    let file1 = Path::new("./README.md").to_path_buf();
    
    let config: Config = Config {
        directory: PathBuf::from("."),
        optional: OptionalConfig {
            markup_types: Some(vec![MarkupType::Html]),
            files: Some(vec![file1]),
            ..Default::default()
        },
    };
    
    let mut result: Vec<MarkupFile> = Vec::new();
    file_traversal::find(&config, &mut result);
    
    // Should not find any files since README.md is markdown, not HTML
    assert_eq!(result.len(), 0);
}

#[test]
fn find_files_with_ignore_path() {
    let file1 = Path::new("./README.md").to_path_buf();
    let ignore_file = std::fs::canonicalize(Path::new("./README.md")).unwrap();
    
    let config: Config = Config {
        directory: PathBuf::from("."),
        optional: OptionalConfig {
            markup_types: Some(vec![MarkupType::Markdown]),
            files: Some(vec![file1]),
            ignore_path: Some(vec![ignore_file]),
            ..Default::default()
        },
    };
    
    let mut result: Vec<MarkupFile> = Vec::new();
    file_traversal::find(&config, &mut result);
    
    // Should be empty because the file is in ignore_path
    assert_eq!(result.len(), 0);
}
