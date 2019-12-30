#[cfg(test)]
use mlc::Config;
use mlc::markup::MarkupType;
use mlc::logger;

#[test]
fn end_to_end() {
    let config = Config {
        folder: String::from("../benches/benchmark"),
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        ignore_links: vec![],
    };
    let _ = mlc::run(&config);
}