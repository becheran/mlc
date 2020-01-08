#[cfg(test)]
use mlc::Config;
use mlc::markup::MarkupType;
use mlc::logger;
use std::path::Path;

#[test]
fn end_to_end() {
    let test_files = Path::new(file!()).parent().unwrap().parent().unwrap().join("benches").join("benchmark");
    let config = Config {
        folder: test_files.to_str().unwrap().to_string(),
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        ignore_links: vec![],
    };
    let _ = mlc::run(&config);
}