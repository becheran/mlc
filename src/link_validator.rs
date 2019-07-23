use crate::link::Link;
use self::url::Url;
use regex::Regex;

extern crate url;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum LinkType {
    HTTP,
    FTP,
    Mail,
    FileSystem,
}

pub fn check(link: &Link) -> Result<String, String> {
    let link_type_opt = get_link_type(&link.target);
    match link_type_opt {
        None =>
            {
                Err(format!("Could not determine link type of {}.", &link.target))
            }
        Some(link_type) =>
            {
                match link_type {
                    LinkType::HTTP | LinkType::FTP | LinkType::Mail
                    => {
                        Err(format!("Link type '{:?}' is not supported yet..", &link_type))
                    }
                    LinkType::FileSystem
                    => {
                        Ok(format!("Link {:?} of type {:?} is valid", link, &link_type))
                    }
                }
            }
    }
}

fn get_link_type(link: &str) -> Option<LinkType> {
    lazy_static! {
        static ref FILE_SYSTEM_REGEX : Regex = Regex::new(
                r"^(([[:alpha:]]:(\\|/))|(..?(\\|/))|((\\\\?|//?))).*"
            ).unwrap();
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

    fn test_links(links: &mut [&str], expected_type: &LinkType) {
        for link in links.iter() {
            let link_type = get_link_type(&link)
                .expect(format!("Unknown link type for link {}", &link).as_str());
            assert_eq!(link_type, *expected_type);
        }
    }

    #[test]
    fn test_http_link_types() {
        let mut links = ["https://doc.rust-lang.org.html", "http://www.website.php"];
        test_links(&mut links, &LinkType::HTTP);
    }


    #[test]
    fn test_ftp_link_types() {
        let mut links = ["ftp://mueller:12345@ftp.downloading.ch"];
        test_links(&mut links, &LinkType::FTP);
    }

    #[test]
    fn mail_link_types() {
        let mut links = ["mailto://name.latname@company.com"];
        test_links(&mut links, &LinkType::Mail);
    }

    #[test]
    fn test_file_system_link_types() {
        let mut links = ["F:/fake/windows/paths", "\\\\smb}\\paths", "C:\\traditional\\paths",
            "\\file.ext", "file:///some/path/", "path", "./file.ext", ".\\file.md",
            "../upper_dir.md", "..\\upper_dir.mdc", "D:\\Program Files(x86)\\file.log",
            "D:\\Program Files(x86)\\folder\\file.log"];
        test_links(&mut links, &LinkType::FileSystem);
    }
}