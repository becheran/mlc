use crate::link::Link;
use self::url::Url;

extern crate url;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum LinkType {
    HTTP,
    FTP,
    MAIL,
    FS,
}

pub fn check<'a>(link: &Link) -> Result<String, String> {
    let link_type_opt = get_link_type(&link.target);
    match link_type_opt {
        None => { return Err(format!("Could not determine link type of {}.", &link.target)); }
        Some(link_type) => {
            match link_type {
                LinkType::HTTP | LinkType::FTP | LinkType::MAIL
                => { return Err(format!("Link type '{:?}' is not supported yet..", &link_type)); }
                LinkType::FS => { return Ok(format!("Link {:?} of type {:?} is valid", link, &link_type)); }
            }
        }
    }
}

fn get_link_type(link: &str) -> Option<LinkType> {
    if let Ok(url) = Url::parse(&link) {
        let scheme = url.scheme();
        debug!("Given link {} is a URL type with scheme {}", link, scheme);
        match scheme {
            "http" => return Some(LinkType::HTTP),
            "https" => return Some(LinkType::HTTP),
            "ftp" => return Some(LinkType::FTP),
            "ftps" => return Some(LinkType::FTP),
            "mailto" => return Some(LinkType::MAIL),
            _ => return None,
        }
    }
    return Some(LinkType::FS);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_link_types() {
        assert_eq!(get_link_type("https://doc.rust-lang.org.html").unwrap(), LinkType::HTTP);
        assert_eq!(get_link_type("http://www.website.php").unwrap(), LinkType::HTTP);
    }
}