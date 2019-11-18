use self::url::Url;
use crate::link::Link;
use crate::link::LinkTrait;
use crate::LinkCheckResult;
use regex::Regex;
use reqwest::Client;
use reqwest::Method;
use reqwest::Request;


extern crate url;

#[derive(Debug, PartialEq)]
pub enum LinkType {
    HTTP,
    FTP,
    Mail,
    FileSystem,
}

pub async fn check(link: &Link) -> LinkCheckResult {
    let link_type_opt = get_link_type(&link.target);
    match link_type_opt {
        None => LinkCheckResult::Failed(format!(
            "Could not determine link type of {}.",
            &link.target
        )),
        Some(link_type) => match link_type {
            LinkType::FTP | LinkType::Mail => LinkCheckResult::NotImplemented(format!(
                "Link type '{:?}' is not supported yet...",
                &link_type
            )),
            LinkType::HTTP => check_http(&link),
            LinkType::FileSystem => check_filesystem(&link),
        },
    }
}

fn check_http(link: &Link) -> LinkCheckResult {
    let client = Client::new();
    let url = reqwest::Url::parse(&link.target).expect("URL of unknown type");
    let request = Request::new(Method::HEAD, url);
    let response = client
        .execute(request)
        .expect("Could not execute http(s) request");
    let status = response.status();
    if status.is_success() {
        LinkCheckResult::Ok(format!("{:?}", link))
    } else {
        LinkCheckResult::Failed(format!(
            "{:?}: Target could not be reached. Status code {:?}",
            &link,
            status.canonical_reason()
        ))
    }
}

fn check_filesystem(link: &Link) -> LinkCheckResult {
    let target = link.absolute_target_path();
    if target.exists() {
        LinkCheckResult::Ok(format!("{:?}", link))
    } else {
        LinkCheckResult::Failed(format!("{:?}: Target path not found.", &link))
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
        debug!("Given link {} is a URL type with scheme {}", link, scheme);
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

    #[test_case("https://doc.rust-lang.org.html")]
    #[test_case("http://www.website.php")]
    fn test_http_link_types(link: &str) {
        test_link(link, &LinkType::HTTP);
    }

    #[test_case("ftp://mueller:12345@ftp.downloading.ch")]
    fn test_ftp_link_types(ftp: &str) {
        test_link(ftp, &LinkType::FTP);
    }

    #[test_case("mailto://name.latname@company.com")]
    #[test_case("mailto://tst@xyz.us")]
    fn mail_link_types(mail: &str) {
        test_link(mail, &LinkType::Mail);
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

    #[test]
    fn check_http_request() {
        let link = Link {
            line_nr: 0,
            target: "http://gitlab.com/becheran/linkchecker".to_string(),
            source: "NotImportant".to_string(),
        };
        let result = check(&link);
        assert!(result.success());
    }

    #[test]
    fn check_https_request() {
        let link = Link {
            line_nr: 0,
            target: "https://gitlab.com/becheran/linkchecker".to_string(),
            source: "NotImportant".to_string(),
        };
        let result = check(&link);
        assert!(result.success());
    }

    #[test]
    #[should_panic]
    fn check_wront_http_request() {
        let link = Link {
            line_nr: 0,
            target: "https://doesNotExist.me/even/less/likelly".to_string(),
            source: "NotImportant".to_string(),
        };
        let result = check(&link);
        assert!(result.success());
    }
}
