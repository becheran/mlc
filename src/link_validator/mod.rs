mod file_system;
mod http;
mod mail;

pub mod link_type;

use crate::Config;
use crate::link_extractors::link_extractor::MarkupLink;
use crate::link_validator::file_system::check_filesystem;
use crate::link_validator::http::check_http;
use log::info;
use mail::check_mail;

pub use link_type::LinkType;
pub use link_type::get_link_type;
use wildmatch::WildMatch;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum LinkCheckResult {
    Ok,
    Failed(String),
    Warning(String),
    Ignored(String),
    NotImplemented(String),
}

/// The target of a link as it is checked, for example the absolute path of a file system link
#[must_use]
pub fn resolve_target_link(
    link: &MarkupLink,
    link_type: &LinkType,
    config: &Config,
    canonical_source: Option<&std::path::Path>,
) -> String {
    if link_type == &LinkType::FileSystem {
        file_system::resolve_target_link(&link.source, canonical_source, &link.target, config)
    } else {
        link.target.clone()
    }
}

pub async fn check(
    link_target: &str,
    link_type: &LinkType,
    config: &Config,
    do_not_warn_for_redirect_to: &[WildMatch],
    http_headers: &[(String, String)],
) -> LinkCheckResult {
    info!("Check link {link_target}.");
    match link_type {
        LinkType::Ftp => LinkCheckResult::NotImplemented(format!(
            "Link type '{link_target:?}' is not supported yet..."
        )),
        LinkType::UnknownUrlSchema | LinkType::Unknown => LinkCheckResult::NotImplemented(
            "Link type is not implemented yet and cannot be checked.".to_string(),
        ),
        LinkType::Mail => check_mail(link_target),
        LinkType::Http => {
            if config.optional.offline.unwrap_or_default() {
                LinkCheckResult::Ignored("Ignore web link because of the offline flag.".to_string())
            } else {
                check_http(link_target, do_not_warn_for_redirect_to, http_headers).await
            }
        }
        LinkType::FileSystem => check_filesystem(link_target, config),
    }
}
