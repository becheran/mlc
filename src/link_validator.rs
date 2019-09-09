use self::url::Url;
use std::path::Path;
use crate::link::Link;
use crate::link::LinkTrait;
use regex::Regex;

extern crate url;

#[derive(Debug, PartialEq)]
pub enum LinkType {
    HTTP,
    FTP,
    Mail,
    FileSystem,
}

pub fn check(link: &Link) -> Result<String, String> {
    let link_type_opt = get_link_type(&link.target);
    match link_type_opt {
        None => Err(format!(
            "Could not determine link type of {}.",
            &link.target
        )),
        Some(link_type) => match link_type {
            LinkType::HTTP | LinkType::FTP | LinkType::Mail => Err(format!(
                "Link type '{:?}' is not supported yet...",
                &link_type
            )),
            LinkType::FileSystem => {
                let target = link.absolute_target_path();
                if target.exists() {
                    Ok(format!("Link {:?} is OK.", link))
                } else {
                    Err(format!("Link target {:?} not found.",target))
                }
            }
        },
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

    #[test]
    fn test_ftp_link_types() {
        test_link("ftp://mueller:12345@ftp.downloading.ch", &LinkType::FTP);
    }

    #[test]
    fn mail_link_types() {
        test_link("mailto://name.latname@company.com", &LinkType::Mail);
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
}
