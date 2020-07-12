mod file_system;
mod http;
mod link_type;
mod mail;

use crate::link_validator::file_system::check_filesystem;
use crate::link_validator::http::check_http;
use crate::Config;
use link_type::get_link_type;
use link_type::LinkType;
use mail::check_mail;

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
        return LinkCheckResult::Ignored("Ignore link because of ignore-links option.".to_string());
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
            LinkType::FileSystem => check_filesystem(link_source, &link_target, &config),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link_validator::LinkCheckResult::Ignored;

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
        match result {
            Ignored(_) => {}
            _ => panic!("Link was not ignored!"),
        }
    }
}
