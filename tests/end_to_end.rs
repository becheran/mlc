use mlc::logger;
use mlc::markup::MarkupType;
#[cfg(test)]
use mlc::Config;
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
        ignore_links: vec![],
        ignore_path: vec!["benches/benchmark/markdown/ignore_me.md".to_string(),"benches/benchmark/markdown/ignore_me".to_string()],
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
        ignore_links: vec![],
        ignore_path: vec![],
        root_dir: Some(test_files),
    };
    if let Err(e) = mlc::run(&config).await {
        panic!("Test with custom root failed. {:?}", e);
    }
}
