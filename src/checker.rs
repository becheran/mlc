//! Checks the distinct link targets of all files

use crate::config::{Config, wildcards};
use crate::link_validator::{self, LinkCheckResult, LinkType};
use log::{info, warn};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::time::{Duration, Instant, sleep_until};
use url::Url;
use wildmatch::WildMatch;

/// A link target which is checked once, no matter how often it is used in the markup files
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Target {
    pub target: String,
    pub link_type: LinkType,
}

pub struct CheckedTarget<'a> {
    pub target: &'a Target,
    pub result: LinkCheckResult,
}

pub struct Checker<'a> {
    config: &'a Config,
    do_not_warn_for_redirect_to: Vec<WildMatch>,
    http_headers: Vec<(String, String)>,
    /// Minimum time between two requests to the same host
    throttle: Option<Duration>,
    /// When the next request to a host is allowed
    next_request: Mutex<HashMap<String, Instant>>,
}

impl<'a> Checker<'a> {
    pub fn new(config: &'a Config) -> Self {
        let http_headers = parse_http_headers(config.optional.http_headers.as_deref());
        info!("Custom HTTP headers: {http_headers:?}");
        let throttle = config
            .optional
            .throttle
            .filter(|&ms| ms > 0)
            .map(|ms| Duration::from_millis(ms.into()));
        info!("Throttle HTTP requests to same host: {throttle:?}");

        Checker {
            config,
            do_not_warn_for_redirect_to: wildcards(
                config.optional.do_not_warn_for_redirect_to.as_deref(),
            ),
            http_headers,
            throttle,
            next_request: Mutex::new(HashMap::new()),
        }
    }

    pub async fn check<'t>(&self, target: &'t Target) -> CheckedTarget<'t> {
        let result = match self.wait_for_turn(target).await {
            Err(failure) => failure,
            Ok(()) => {
                link_validator::check(
                    &target.target,
                    &target.link_type,
                    self.config,
                    &self.do_not_warn_for_redirect_to,
                    &self.http_headers,
                )
                .await
            }
        };
        CheckedTarget { target, result }
    }

    /// Delays web requests, so the same host is not asked more often than the throttle allows
    async fn wait_for_turn(&self, target: &Target) -> Result<(), LinkCheckResult> {
        let Some(throttle) = self.throttle.filter(|_| target.link_type == LinkType::Http) else {
            return Ok(());
        };
        let url = Url::parse(&target.target).map_err(|error| {
            LinkCheckResult::Failed(format!("Could not parse URL type. Err: {error:?}"))
        })?;
        let host = url
            .host_str()
            .ok_or_else(|| LinkCheckResult::Failed("Failed to determine host".to_string()))?;

        let turn = {
            let mut next_request = self.next_request.lock().unwrap();
            let turn = next_request.get(host).copied();
            // The first request does not wait at all, every following one waits for its slot
            let following = turn.unwrap_or_else(Instant::now) + throttle;
            next_request.insert(host.to_string(), following);
            turn
        };
        if let Some(deadline) = turn {
            sleep_until(deadline).await;
        }
        Ok(())
    }
}

/// Parses headers in the format `Name: Value`
fn parse_http_headers(headers: Option<&[String]>) -> Vec<(String, String)> {
    headers
        .unwrap_or_default()
        .iter()
        .filter_map(|header| match header.split_once(':') {
            Some((name, value)) => Some((name.trim().to_string(), value.trim().to_string())),
            None => {
                warn!("Invalid HTTP header format (expected 'Name: Value'): {header}");
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_headers_are_split_at_first_colon() {
        let headers = [
            "User-Agent: Mozilla/5.0".to_string(),
            "Referer:https://example.com:8080/".to_string(),
            "broken".to_string(),
        ];
        assert_eq!(
            parse_http_headers(Some(&headers)),
            vec![
                ("User-Agent".to_string(), "Mozilla/5.0".to_string()),
                (
                    "Referer".to_string(),
                    "https://example.com:8080/".to_string()
                ),
            ]
        );
    }

    #[test]
    fn no_http_headers() {
        assert!(parse_http_headers(None).is_empty());
    }
}
