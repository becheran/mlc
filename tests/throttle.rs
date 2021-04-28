#[cfg(test)]
mod helper;

use helper::benches_dir;
use mlc::logger;
use mlc::markup::MarkupType;
use mlc::Config;
use std::time::{Duration, Instant};


const TEST_THROTTLE_MS: u32 = 100;
const TEST_URLS: u32 = 10;

#[tokio::test]
async fn throttle_different_hosts() {
    let test_file = benches_dir().join("throttle").join("different_host.md");
    let config = Config {
        folder: test_file,
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        match_file_extension: false,
        throttle: TEST_THROTTLE_MS,
        ignore_links: vec![],
        ignore_path: vec![],
        root_dir: None,
    };

    let start = Instant::now();
    mlc::run(&config).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration < Duration::from_millis(TEST_THROTTLE_MS.into()) * TEST_URLS / 2)
}

#[tokio::test]
async fn throttle_same_hosts() {
    let test_file = benches_dir().join("throttle").join("same_host.md");
    let config = Config {
        folder: test_file,
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        match_file_extension: false,
        throttle: TEST_THROTTLE_MS,
        ignore_links: vec![],
        ignore_path: vec![],
        root_dir: None,
    };

    let start = Instant::now();
    mlc::run(&config).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration > Duration::from_millis(TEST_THROTTLE_MS.into()) * TEST_URLS / 2)
}

#[tokio::test]
async fn throttle_same_ip() {
    let test_file = benches_dir().join("throttle").join("same_ip.md");
    let config = Config {
        folder: test_file,
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
        no_web_links: false,
        match_file_extension: false,
        throttle: TEST_THROTTLE_MS,
        ignore_links: vec![],
        ignore_path: vec![],
        root_dir: None,
    };

    let start = Instant::now();
    mlc::run(&config).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration > Duration::from_millis(TEST_THROTTLE_MS.into()) * TEST_URLS / 2)
}
