use crate::link_validator::LinkCheckResult;

use reqwest::header::ACCEPT;
use reqwest::header::USER_AGENT;
use reqwest::redirect;
use reqwest::Client;
use reqwest::Method;
use reqwest::Request;
use reqwest::StatusCode;
use wildmatch::WildMatch;

pub async fn check_http(
    target: &str,
    do_not_warn_for_redirect_to: &Vec<WildMatch>,
) -> LinkCheckResult {
    debug!("Check http link target {:?}", target);
    let url = reqwest::Url::parse(target).expect("URL of unknown type");

    match http_request(&url, do_not_warn_for_redirect_to).await {
        Ok(response) => response,
        Err(error_msg) => LinkCheckResult::Failed(format!("Http(s) request failed. {}", error_msg)),
    }
}

fn new_request(method: Method, url: &reqwest::Url) -> Request {
    let mut req = Request::new(method, url.clone());
    let headers = req.headers_mut();
    headers.insert(ACCEPT, "text/html, text/markdown".parse().unwrap());
    headers.insert(USER_AGENT, "mlc (github.com/becheran/mlc)".parse().unwrap());
    req
}

async fn http_request(
    url: &reqwest::Url,
    do_not_warn_for_redirect_to: &Vec<WildMatch>,
) -> reqwest::Result<LinkCheckResult> {
    lazy_static! {
        static ref CLIENT: Client = reqwest::Client::builder()
            .brotli(true)
            .gzip(true)
            .deflate(true)
            .redirect(redirect::Policy::custom(|attempt| {
                if attempt.previous().len() > 10 {
                    attempt.error("too many redirects")
                } else if do_not_warn_for_redirect_to
                    .iter()
                    .any(|x| x.matches(attempt.url().as_ref()))
                {
                    attempt.stop()
                } else {
                    attempt.follow()
                }
            }))
            .build()
            .expect("Bug! failed to build client");
    }

    fn status_to_string(status: StatusCode) -> String {
        format!(
            "{} - {}",
            status.as_str(),
            status.canonical_reason().unwrap_or("Unknown reason")
        )
    }

    let head_request = new_request(Method::HEAD, url);
    let get_request = new_request(Method::GET, url);

    let response = match CLIENT.execute(head_request).await {
        Ok(r) => r,
        Err(e) => {
            println!("Head request error: {}. Retry with get-request.", e);
            CLIENT.execute(get_request).await?
        }
    };

    let status = response.status();
    if status.is_success() {
        if response.url() == url
            || do_not_warn_for_redirect_to
                .iter()
                .any(|x| x.matches(response.url().as_ref()))
        {
            Ok(LinkCheckResult::Ok)
        } else {
            Ok(LinkCheckResult::Warning(
                "Request was redirected to ".to_string() + response.url().as_ref(),
            ))
        }
    } else if status.is_redirection() {
        // Only if > 10 redirects
        Ok(LinkCheckResult::Warning(status_to_string(status)))
    } else {
        debug!("Got the status code {:?}. Retry with get-request.", status);
        let get_request = Request::new(Method::GET, url.clone());
        let response = CLIENT.execute(get_request).await?;
        let status = response.status();
        if status.is_success() {
            if response.url() == url
                || do_not_warn_for_redirect_to
                    .iter()
                    .any(|x| x.matches(response.url().as_ref()))
            {
                Ok(LinkCheckResult::Ok)
            } else {
                Ok(LinkCheckResult::Warning(status_to_string(status)))
            }
        } else {
            Ok(LinkCheckResult::Failed(status_to_string(status)))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::logger;

    use super::*;

    #[tokio::test]
    async fn check_http_is_available() {
        let result = check_http("https://www.google.com/", &vec![]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_is_redirection() {
        let result = check_http("http://google.com", &vec![]).await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(
                "Request was redirected to http://www.google.com/".to_string()
            )
        );
    }

    #[tokio::test]
    async fn check_http_redirection_do_not_warn_if_ignored() {
        // we ignore redirections to the 'https'-version
        let result = check_http(
            "http://gitlab.com/becheran/mlc",
            &vec![WildMatch::new("https://gitlab.com/becheran/mlc")],
        )
        .await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_redirection_do_not_warn_if_ignored_star_pattern() {
        let result = check_http("http://www.google.com", &vec![WildMatch::new("*")]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_redirection_do_warn_if_ignored_mismatch() {
        logger::init(&logger::LogLevel::Debug);
        let result = check_http(
            "http://amazon.com",
            &vec![WildMatch::new("http://google.com")],
        )
        .await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(
                "Request was redirected to https://www.amazon.de/".to_string()
            )
        );
    }

    #[tokio::test]
    async fn check_http_is_redirection_failure() {
        let result = check_http("http://gitlab.com/fake-page/does/not/exist/ever", &vec![]).await;
        assert_eq!(
            result,
            LinkCheckResult::Failed("403 - Forbidden".to_string())
        );
    }

    #[tokio::test]
    async fn check_https_crates_io_available() {
        let result = check_http("https://crates.io", &vec![]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_request_with_hash() {
        let result = check_http("https://www.google.com/#bla", &vec![]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_request_redirection_with_hash() {
        let result = check_http("https://google.com#bla", &vec![]).await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(
                "Request was redirected to https://www.google.com/".to_string()
            )
        );
    }

    #[tokio::test]
    async fn check_wrong_http_request() {
        let result = check_http("https://doesNotExist.me/even/less/likelly", &vec![]).await;
        assert!(result != LinkCheckResult::Ok);
    }
}
