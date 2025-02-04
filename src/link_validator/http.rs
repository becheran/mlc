use crate::link_validator::LinkCheckResult;

use reqwest::header::ACCEPT;
use reqwest::header::USER_AGENT;
use reqwest::Client;
use reqwest::Method;
use reqwest::Request;
use reqwest::StatusCode;
use wildmatch::WildMatch;

pub async fn check_http(
    target: &str,
    do_not_warn_for_redirect_to: &[WildMatch],
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
    do_not_warn_for_redirect_to: &[WildMatch],
) -> reqwest::Result<LinkCheckResult> {
    lazy_static! {
        static ref CLIENT: Client = reqwest::Client::builder()
            .brotli(true)
            .gzip(true)
            .deflate(true)
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

    let response = CLIENT.execute(new_request(Method::HEAD, url)).await?;
    let check_redirect = |response_url: &reqwest::Url| -> reqwest::Result<LinkCheckResult> {
        if response_url == url
            || do_not_warn_for_redirect_to
                .iter()
                .any(|x| x.matches(response_url.as_ref()))
        {
            Ok(LinkCheckResult::Ok)
        } else {
            Ok(LinkCheckResult::Warning(
                "Request was redirected to ".to_string() + response_url.as_ref(),
            ))
        }
    };

    let status = response.status();
    if status.is_success() || status.is_redirection() {
        check_redirect(response.url())
    } else {
        debug!("Got the status code {:?}. Retry with get-request.", status);
        let get_request = Request::new(Method::GET, url.clone());

        let response = CLIENT.execute(get_request).await?;
        let status = response.status();
        if status.is_success() || status.is_redirection() {
            check_redirect(response.url())
        } else {
            Ok(LinkCheckResult::Failed(status_to_string(status)))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn check_http_is_available() {
        let result = check_http("https://gitlab.com/becheran/mlc", &[]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_is_redirection() {
        let result = check_http("http://gitlab.com/becheran/mlc", &[]).await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(
                "Request was redirected to https://gitlab.com/becheran/mlc".to_string()
            )
        );
    }

    #[tokio::test]
    async fn check_http_redirection_do_not_warn_if_ignored() {
        // we ignore redirections to the 'https'-version
        let result = check_http(
            "http://gitlab.com/becheran/mlc",
            &[WildMatch::new("https://gitlab.com/becheran/mlc")],
        )
        .await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_redirection_do_not_warn_if_ignored_star_pattern() {
        let result = check_http("http://gitlab.com/becheran/mlc", &[WildMatch::new("*")]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_redirection_do_warn_if_ignored_mismatch() {
        let result = check_http(
            "http://gitlab.com/becheran/mlc",
            &[WildMatch::new("http://www.google.com")],
        )
        .await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(
                "Request was redirected to https://gitlab.com/becheran/mlc".to_string()
            )
        );
    }

    #[tokio::test]
    async fn check_http_is_redirection_failure() {
        let result = check_http("http://gitlab.com/fake-page/does/not/exist/ever", &[]).await;
        assert_eq!(
            result,
            LinkCheckResult::Failed("403 - Forbidden".to_string())
        );
    }

    #[tokio::test]
    async fn check_https_crates_io_available() {
        let result = check_http("https://crates.io", &[]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_request_with_hash() {
        let result = check_http("https://gitlab.com/becheran/mlc#bla", &[]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_request_redirection_with_hash() {
        let result = check_http("http://gitlab.com/becheran/mlc#bla", &[]).await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(
                "Request was redirected to https://gitlab.com/becheran/mlc".to_string()
            )
        );
    }

    #[tokio::test]
    async fn check_wrong_http_request() {
        let result = check_http("https://doesNotExist.me/even/less/likelly", &[]).await;
        assert!(result != LinkCheckResult::Ok);
    }
}
