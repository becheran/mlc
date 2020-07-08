mod link_type;
mod mail;

use mail::check_mail;
use link_type::LinkType;
use link_type::get_link_type;
use crate::Config;
use reqwest::Client;
use reqwest::Method;
use reqwest::Request;
use reqwest::StatusCode;
use std::path::Path;
use std::path::PathBuf;
use std::path::MAIN_SEPARATOR;


#[derive(Debug, PartialEq, Clone)]
pub enum LinkCheckResult {
    Ok,
    Failed(String),
    Warning(String),
    Ignored(String),
    NotImplemented(String),
}

pub async fn check(link_source: &str, link_target: &str, config: &Config) -> LinkCheckResult {
    info!("Check link {} => {}.", &link_source, &link_target);
    if config.ignore_links.iter().any(|m| m.is_match(link_target)) {
        return LinkCheckResult::Ignored(
            "Ignore web link because of ignore-links option.".to_string(),
        );
    }
    match get_link_type(link_target) {
        None => {
            LinkCheckResult::Failed(format!("Could not determine link type of {}.", link_target))
        }
        Some(link_type) => match link_type {
            LinkType::FTP => LinkCheckResult::NotImplemented(format!(
                "Link type '{:?}' is not supported yet...",
                &link_type
            )),
            LinkType::UnknownUrlSchema => LinkCheckResult::NotImplemented(
                "Url schema is unknown and cannot be checked.".to_string(),
            ),
            LinkType::Mail => check_mail(link_target),
            LinkType::HTTP => {
                if config.no_web_links {
                    LinkCheckResult::Ignored(
                        "Ignore web link because of the no-web-link flag.".to_string(),
                    )
                } else {
                    check_http(link_target).await
                }
            }
            LinkType::FileSystem => {
                let mut normalized_link = link_target
                    .replace('/', &MAIN_SEPARATOR.to_string())
                    .replace('\\', &MAIN_SEPARATOR.to_string());
                if let Some(idx) = normalized_link.find('#') {
                    warn!(
                        "Strip everything after #. The chapter part ´{}´ is not checked.",
                        &normalized_link[idx..]
                    );
                    normalized_link = normalized_link[..idx].to_string();
                }
                let mut fs_link_target = Path::new(&normalized_link).to_path_buf();
                if normalized_link.starts_with(MAIN_SEPARATOR) && config.root_dir.is_some() {
                    match std::fs::canonicalize(&config.root_dir.as_ref().unwrap())
                    {
                        Ok(new_root) => fs_link_target = new_root.join(Path::new(&normalized_link[1..])),
                        Err(e) => panic!("Root path could not be converted to an absolute path. Does the directory exit? {}", e)
                    }
                }
                check_filesystem(link_source, &fs_link_target)
            }
        },
    }
}

async fn http_request(url: &reqwest::Url) -> reqwest::Result<LinkCheckResult> {
    lazy_static! {
        static ref CLIENT: Client = Client::new();
    }

    fn status_to_string(status: &StatusCode) -> String {
        format!(
            "{} - {}",
            status.as_str(),
            status.canonical_reason().unwrap_or("Unknown reason")
        )
    }

    let head_request = Request::new(Method::HEAD, url.clone());
    let get_request = Request::new(Method::GET, url.clone());

    let response = match CLIENT.execute(head_request).await {
        Ok(r) => r,
        Err(e) => {
            println!("Head request error: {}. Retry with get-request.", e);
            CLIENT.execute(get_request).await?
        }
    };

    let status = response.status();
    if status.is_success() {
        Ok(LinkCheckResult::Ok)
    } else if status.is_redirection() {
        Ok(LinkCheckResult::Warning(status_to_string(&status)))
    } else if status == reqwest::StatusCode::METHOD_NOT_ALLOWED
        || status == reqwest::StatusCode::BAD_REQUEST
    {
        debug!("Got the status code {:?}. Retry with get-request.", status);
        let get_request = Request::new(Method::GET, url.clone());
        let response = CLIENT.execute(get_request).await?;
        let status = response.status();
        if status.is_success() {
            Ok(LinkCheckResult::Ok)
        } else {
            Ok(LinkCheckResult::Failed(status_to_string(&status)))
        }
    } else {
        Ok(LinkCheckResult::Failed(status_to_string(&status)))
    }
}

async fn check_http(target: &str) -> LinkCheckResult {
    debug!("Check http link target {:?}", target);
    let url = reqwest::Url::parse(&target).expect("URL of unknown type");

    match http_request(&url).await {
        Ok(response) => response,
        Err(error_msg) => LinkCheckResult::Failed(format!("Http(s) request failed. {}", error_msg)),
    }
}

fn check_filesystem(source: &str, target: &PathBuf) -> LinkCheckResult {
    debug!("Check file system link target {:?}", target);
    let target = absolute_target_path(source, target);
    debug!("Absolute target path {:?}", target);
    if target.exists() {
        LinkCheckResult::Ok
    } else {
        LinkCheckResult::Failed("Target path not found.".to_string())
    }
}

fn absolute_target_path(source: &str, target: &PathBuf) -> PathBuf {
    if target.is_relative() {
        let parent = Path::new(source).parent().unwrap_or(Path::new("./"));
        parent.join(target)
    } else {
        target.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn check_http_request() {
        let config = Config::default();
        let result = check("NotImportant", "http://gitlab.com/becheran/mlc", &config).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_request_with_hash() {
        let config = Config::default();
        let result = check(
            "NotImportant",
            "http://gitlab.com/becheran/mlc#bla",
            &config,
        )
        .await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_https_request() {
        let config = Config::default();
        let result = check("NotImportant", "https://gitlab.com/becheran/mlc", &config).await;
        assert!(result == LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_wrong_http_request() {
        let config = Config::default();
        let result = check(
            "NotImportant",
            "https://doesNotExist.me/even/less/likelly",
            &config,
        )
        .await;
        assert!(result != LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn ignore_link_pattern() {
        let mut config = Config::default();
        config.ignore_links = vec![wildmatch::WildMatch::new("http?*")];
        let result = check(
            "NotImportant",
            "https://doesNotExist.me/even/less/likelly",
            &config,
        )
        .await;
        assert_eq!(
            result,
            LinkCheckResult::Ignored("Ignore web link because of ignore-links option.".to_string())
        );
    }
}
