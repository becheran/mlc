#[cfg(test)]
use linkchecker::file_traversal;
use std::env;
use std::path::{PathBuf, Path};

fn root_dir() -> String {
    let default_path = PathBuf::from(r"/");
    let path = env::current_dir().unwrap_or(default_path).to_string_lossy().to_string();
    path
}

#[test]
fn find_markdown_files() {
    let mut path = root_dir();
    let file_extension = [".md".to_string()];
    path.push_str("/tests/traversal_test/three_empty_md_files");
    let mut result: Vec<String> = Vec::new();
    file_traversal::find(&path, &file_extension, &mut result);
    assert_eq!(result.len(), 3);
    let possible_results = ["f1.md", "f2.MD", "F3_with_umlaut.md"];
    for r in result {
        assert!(possible_results.contains(&Path::new(&r).file_name().unwrap().to_str().unwrap()));
    }
}

#[test]
fn empty_folder() {
    let mut path = root_dir();
    let file_extension = [".md".to_string()];
    path.push_str("/tests/traversal_test/empty");
    let mut result: Vec<String> = Vec::new();
    file_traversal::find(&path, &file_extension, &mut result);
    assert!(result.is_empty());
}
