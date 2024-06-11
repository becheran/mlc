#[cfg(test)]
mod helper;

use helper::benches_dir;
use mlc::{markup::MarkupType, Config, OptionalConfig};
use std::time::{Duration, Instant};

const TEST_THROTTLE_MS: u32 = 100;
const TEST_URLS: u32 = 10;
const THROTTLED_TIME_MS: u64 = (TEST_THROTTLE_MS as u64) * ((TEST_URLS as u64) - 1);

#[tokio::test]
async fn throttle_different_hosts() {
    let config = Config {
        directory: benches_dir().join("throttle").join("different_host.md"),
        optional: OptionalConfig {
            throttle: Some(TEST_THROTTLE_MS),
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    };
    let start = Instant::now();
    mlc::run(&config).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration < Duration::from_millis(THROTTLED_TIME_MS))
}

#[tokio::test]
async fn throttle_same_hosts() {
    let config = Config {
        directory: benches_dir().join("throttle").join("same_host.md"),
        optional: OptionalConfig {
            throttle: Some(TEST_THROTTLE_MS),
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    };

    let start = Instant::now();
    mlc::run(&config).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration > Duration::from_millis(THROTTLED_TIME_MS))
}

#[tokio::test]
async fn throttle_same_ip() {
    let config = Config {
        directory: benches_dir().join("throttle").join("same_ip.md"),
        optional: OptionalConfig {
            throttle: Some(TEST_THROTTLE_MS),
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    };

    let start = Instant::now();
    mlc::run(&config).await.unwrap_or(());
    let duration = start.elapsed();
    assert!(duration > Duration::from_millis(THROTTLED_TIME_MS))
}
