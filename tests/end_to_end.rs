#[cfg(test)]
use linkchecker::Config;
use linkchecker::markup::MarkupType;
use linkchecker::logger;

#[test]
fn end_to_end() {
    let config = Config {
        folder: String::from("./tests/benchmark"),
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
    };
    let _ = linkchecker::run(&config);
}