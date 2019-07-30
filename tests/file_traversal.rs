#[cfg(test)]
use linkchecker::file_traversal;
use std::env;
use std::path::{PathBuf, Path};
use linkchecker::Config;
use linkchecker::markup::{MarkupType, MarkupFile};

fn root_dir() -> String {
    let default_path = PathBuf::from(r"/");
    let path = env::current_dir().unwrap_or(default_path).to_string_lossy().to_string();
    path
}

#[test]
fn find_markdown_files() {
    let mut path = root_dir();
    path.push_str("/tests/benchmark/three_empty_md_files");
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
    let mut path = root_dir();
    path.push_str("/tests/benchmark/empty");
    let config: Config = Config {
        folder: path,
        markup_types: vec![MarkupType::Markdown],
        ..Default::default()
    };
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config, &mut result);
    assert!(result.is_empty());
}
