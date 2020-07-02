use self::url::Url;
use crate::Config;
use regex::Regex;
use reqwest::Client;
use reqwest::Method;
use reqwest::Request;
use reqwest::StatusCode;
use std::path::Path;
use std::path::PathBuf;

extern crate url;

#[derive(Debug, PartialEq)]
pub enum LinkType {
    HTTP,
    FTP,
    Mail,
    FileSystem,
}

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
    let link_type_opt = get_link_type(link_target);
    match link_type_opt {
        None => {
            LinkCheckResult::Failed(format!("Could not determine link type of {}.", link_target))
        }
        Some(link_type) => match link_type {
            LinkType::FTP => LinkCheckResult::NotImplemented(format!(
                "Link type '{:?}' is not supported yet...",
                &link_type
            )),
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
                let mut fs_link_target = Path::new(link_target).to_path_buf();
                if (link_target.starts_with('/') || link_target.starts_with('\\')) && config.root_dir.is_some() {
                    match std::fs::canonicalize(&config.root_dir.as_ref().unwrap())
                    {
                        Ok(new_root) => fs_link_target = new_root.join(Path::new(&link_target[1..])),
                        Err(e) => panic!("Root path could not be converted to an absolute path. Does the directory exit? {}", e)
                    }
                }
                check_filesystem(link_source, &fs_link_target)
            }
        },
    }
}

fn check_mail(target: &str) -> LinkCheckResult {
    lazy_static! {
        static ref EMAIL_REGEX: Regex = Regex::new(
            r"^mailto:(//)?([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
        )
        .unwrap();
    }
    if EMAIL_REGEX.is_match(target) {
        LinkCheckResult::Ok
    } else {
        LinkCheckResult::Failed(format!("Not a valid mail adress."))
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
    let url = reqwest::Url::parse(&target).expect("URL of unknown type");

    match http_request(&url).await {
        Ok(response) => response,
        Err(error_msg) => LinkCheckResult::Failed(format!("Http(s) request failed. {}", error_msg)),
    }
}

fn check_filesystem(source: &str, target: &PathBuf) -> LinkCheckResult {
    let target = absolute_target_path(source, target);
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

fn get_link_type(link: &str) -> Option<LinkType> {
    lazy_static! {
        static ref FILE_SYSTEM_REGEX: Regex =
            Regex::new(r"^(([[:alpha:]]:(\\|/))|(..?(\\|/))|((\\\\?|//?))).*").unwrap();
    }

    if FILE_SYSTEM_REGEX.is_match(link) || !link.contains(':') {
        return Some(LinkType::FileSystem);
    }

    if let Ok(url) = Url::parse(&link) {
        let scheme = url.scheme();
        debug!("Link {} is a URL type with scheme {}", link, scheme);
        match scheme {
            "http" => return Some(LinkType::HTTP),
            "https" => return Some(LinkType::HTTP),
            "ftp" => return Some(LinkType::FTP),
            "ftps" => return Some(LinkType::FTP),
            "mailto" => return Some(LinkType::Mail),
            "file" => return Some(LinkType::FileSystem),
            _ => return None,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;

    fn test_link(link: &str, expected_type: &LinkType) {
        let link_type =
            get_link_type(link).expect(format!("Unknown link type for link {}", link).as_str());
        assert_eq!(link_type, *expected_type);
    }

    #[test_case("mailto://+bar@bar.com")]
    #[test_case("mailto://foo+@bar.com")]
    #[test_case("mailto://foo.lastname@bar.com")]
    #[test_case("mailto://tst@xyz.us")]
    #[test_case("mailto:bla.bla@web.de")]
    fn mail_links(link: &str) {
        let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        let config = Config::default();
        let result = runtime.block_on(check("NotImportant", link, &config));
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[test_case("mailto://+bar@bar")]
    #[test_case("mailto://foobar.com")]
    #[test_case("mailto://foo.lastname.com")]
    fn invalid_mail_links(link: &str) {
        let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        let config = Config::default();
        let result = runtime.block_on(check("NotImportant", link, &config));
        assert!(result != LinkCheckResult::Ok);
    }

    #[test_case("https://doc.rust-lang.org.html")]
    #[test_case("http://www.website.php")]
    fn http_link_types(link: &str) {
        test_link(link, &LinkType::HTTP);
    }

    #[test_case("ftp://mueller:12345@ftp.downloading.ch")]
    fn ftp_link_types(ftp: &str) {
        test_link(ftp, &LinkType::FTP);
    }

    #[test_case("F:/fake/windows/paths")]
    #[test_case("\\\\smb}\\paths")]
    #[test_case("C:\\traditional\\paths")]
    #[test_case("\\file.ext")]
    #[test_case("file:///some/path/")]
    #[test_case("path")]
    #[test_case("./file.ext")]
    #[test_case(".\\file.md")]
    #[test_case("../upper_dir.md")]
    #[test_case("..\\upper_dir.mdc")]
    #[test_case("D:\\Program Files(x86)\\file.log")]
    #[test_case("D:\\Program Files(x86)\\folder\\file.log")]
    fn test_file_system_link_types(link: &str) {
        test_link(link, &LinkType::FileSystem);
    }

    #[tokio::test]
    async fn check_http_request() {
        let config = Config::default();
        let result = check("NotImportant", "http://gitlab.com/becheran/mlc", &config).await;
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
