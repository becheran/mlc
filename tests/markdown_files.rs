#[cfg(test)]
use linkchecker::link_extractor;
use std::env;
use std::path::PathBuf;
use linkchecker::markup::{MarkupFile, MarkupType};

fn root_dir() -> String {
    let default_path = PathBuf::from(r"/");
    let path = env::current_dir().unwrap_or(default_path).to_string_lossy().to_string();
    path
}

#[test]
fn no_links() {
    let mut path = root_dir();
    path.push_str("/tests/benchmark/no_links/no_links.md");
    let file = MarkupFile {
        path,
        markup_type: MarkupType::Markdown,
    };
    let result = link_extractor::find_links(&file);
    assert!(result.is_empty());
}

#[test]
fn some_links() {
    let mut path = root_dir();
    path.push_str("/tests/benchmark/many_links/many_links.md");
    let file = MarkupFile {
        path,
        markup_type: MarkupType::Markdown,
    };
    let result = link_extractor::find_links(&file);
    assert_eq!(result.len(), 10);
}
