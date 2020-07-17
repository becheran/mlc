use mlc::logger;
use mlc::markup::MarkupType;
#[cfg(test)]
use mlc::Config;
use std::fs;
use std::path::Path;

#[tokio::test]
async fn end_to_end() {
    let test_files = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("benches")
        .join("benchmark");
    let config = Config {
        folder: test_files,
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        match_file_extension: false,
        throttle: 0,
        ignore_links: vec![],
        ignore_path: vec![
            fs::canonicalize("benches/benchmark/markdown/ignore_me.md").unwrap(),
            fs::canonicalize("./benches/benchmark/markdown/ignore_me_dir").unwrap(),
        ],
        root_dir: None,
    };
    if let Err(_) = mlc::run(&config).await {
        panic!();
    }
}

#[tokio::test]
async fn end_to_end_different_root() {
    let test_files = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("benches")
        .join("different_root");
    let config = Config {
        folder: test_files.clone(),
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        match_file_extension: false,
        ignore_links: vec![],
        ignore_path: vec![],
        throttle: 0,
        root_dir: Some(test_files),
    };
    if let Err(e) = mlc::run(&config).await {
        panic!("Test with custom root failed. {:?}", e);
    }
}
