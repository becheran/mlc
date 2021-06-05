use crate::link_validator::LinkCheckResult;
use regex::Regex;

pub fn check_mail(target: &str) -> LinkCheckResult {
    debug!("Check mail target {:?}", target);
    let mut mail = target;
    if let Some(stripped) = target.strip_prefix("mailto://") {
        mail = stripped;
    } else if let Some(stripped) = target.strip_prefix("mailto:") {
        mail = stripped;
    }
    lazy_static! {
        static ref EMAIL_REGEX: Regex = Regex::new(
            r"^((?i)[a-z0-9_!#$%&'*+-/=?^`{|}~+]([a-z0-9_!#$%&'*+-/=?^`{|}~+.]*[a-z0-9_!#$%&'*+-/=?^_{|}~+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
        )
        .unwrap();
    }
    if EMAIL_REGEX.is_match(mail) {
        LinkCheckResult::Ok
    } else {
        LinkCheckResult::Failed("Not a valid mail address.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;

    #[test_case("mailto://+bar@bar.com")]
    #[test_case("mailto://foo+@bar.com")]
    #[test_case("mailto://foo.lastname@bar.com")]
    #[test_case("mailto://tst@xyz.us")]
    #[test_case("mailto:bla.bla@web.de")]
    #[test_case("mailto:bla.bla.ext@web.de")]
    #[test_case("mailto:BlA.bLa.ext@web.de")]
    #[test_case("mailto:foo-bar@foobar.com")]
    #[test_case("mailto:!#$%&'*+-/=?^_`{|}~-foo@foobar.com")]
    #[test_case("mailto:some@hostnumbers123.com")]
    #[test_case("mailto:some@host-name.com")]
    #[test_case("bla.bla@web.de")]
    fn mail_links(link: &str) {
        let result = check_mail(link);
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[test_case("mailto://@bar@bar")]
    #[test_case("mailto://foobar.com")]
    #[test_case("mailto://foo.lastname.com")]
    #[test_case("mailto:foo.do@l$astname.cOM")]
    #[test_case("mailto:foo@l_astname.cOM")]
    fn invalid_mail_links(link: &str) {
        let result = check_mail(link);
        assert!(result != LinkCheckResult::Ok);
    }
}
