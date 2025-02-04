mod file_system;
mod http;
mod mail;

pub mod link_type;

use crate::link_extractors::link_extractor::MarkupLink;
use crate::link_validator::file_system::check_filesystem;
use crate::link_validator::http::check_http;
use crate::Config;
use mail::check_mail;

pub use link_type::get_link_type;
pub use link_type::LinkType;
use wildmatch::WildMatch;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LinkCheckResult {
    Ok,
    Failed(String),
    Warning(String),
    Ignored(String),
    NotImplemented(String),
}

pub async fn resolve_target_link(
    link: &MarkupLink,
    link_type: &LinkType,
    config: &Config,
) -> String {
    if link_type == &LinkType::FileSystem {
        file_system::resolve_target_link(&link.source, &link.target, config).await
    } else {
        link.target.to_string()
    }
}

pub async fn check(
    link_target: &str,
    link_type: &LinkType,
    config: &Config,
    do_not_warn_for_redirect_to: &[WildMatch],
) -> LinkCheckResult {
    info!("Check link {}.", &link_target);
    match link_type {
        LinkType::Ftp => LinkCheckResult::NotImplemented(format!(
            "Link type '{:?}' is not supported yet...",
            &link_target
        )),
        LinkType::UnknownUrlSchema => LinkCheckResult::NotImplemented(
            "Link type is not implemented yet and cannot be checked.".to_string(),
        ),
        LinkType::Mail => check_mail(link_target),
        LinkType::Http => {
            if config.optional.offline.unwrap_or_default() {
                LinkCheckResult::Ignored("Ignore web link because of the offline flag.".to_string())
            } else {
                check_http(link_target, do_not_warn_for_redirect_to).await
            }
        }
        LinkType::FileSystem => check_filesystem(link_target, config).await,
    }
}
