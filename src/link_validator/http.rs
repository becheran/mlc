use crate::link_validator::LinkCheckResult;

use reqwest::header::ACCEPT;
use reqwest::header::USER_AGENT;
use reqwest::Client;
use reqwest::Method;
use reqwest::Request;
use reqwest::StatusCode;
use wildmatch::WildMatch;

const BROWSER_ACCEPT_HEADER: &str =
    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";

pub async fn check_http(
    target: &str,
    do_not_warn_for_redirect_to: &[WildMatch],
) -> LinkCheckResult {
    debug!("Check http link target {target:?}");
    let url = reqwest::Url::parse(target).expect("URL of unknown type");

    match http_request(&url, do_not_warn_for_redirect_to).await {
        Ok(response) => response,
        Err(error_msg) => LinkCheckResult::Failed(format!("Http(s) request failed. {error_msg}")),
    }
}

fn new_request(method: Method, url: &reqwest::Url) -> Request {
    let mut req = Request::new(method, url.clone());
    let headers = req.headers_mut();
    headers.insert(ACCEPT, BROWSER_ACCEPT_HEADER.parse().unwrap());
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
        // Compare URLs ignoring fragments since fragments are not sent to the server
        // and the response URL will never have them
        let urls_match = url.scheme() == response_url.scheme()
            && url.host() == response_url.host()
            && url.port() == response_url.port()
            && url.path() == response_url.path()
            && url.query() == response_url.query();

        if urls_match
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
        debug!("Got the status code {status:?}. Retry with get-request.");
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
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;

        let result = check_http(&server.url(), &[]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_fail() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(500)
            .create_async()
            .await;

        let result = check_http(&server.url(), &[]).await;
        assert_eq!(
            result,
            LinkCheckResult::Failed("500 - Internal Server Error".to_string())
        );
    }

    #[tokio::test]
    async fn check_http_is_redirection() {
        let mut redirect_server = mockito::Server::new_async().await;
        redirect_server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;

        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(301)
            .with_header("Location", &redirect_server.url())
            .create_async()
            .await;

        let result = check_http(&server.url(), &[]).await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(format!(
                "Request was redirected to {}/",
                &redirect_server.url()
            ))
        );
    }

    #[tokio::test]
    async fn check_http_redirection_do_not_warn_if_ignored() {
        let mut redirect_server = mockito::Server::new_async().await;
        redirect_server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(301)
            .with_header("Location", &redirect_server.url())
            .create_async()
            .await;

        let result = check_http(
            &server.url(),
            &[WildMatch::new(&format!("{}*", &redirect_server.url()))],
        )
        .await;

        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_redirection_do_not_warn_if_ignored_star_pattern() {
        let mut redirect_server = mockito::Server::new_async().await;
        redirect_server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(301)
            .with_header("Location", &redirect_server.url())
            .create_async()
            .await;

        let result = check_http(&server.url(), &[WildMatch::new("*")]).await;

        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_redirection_do_warn_if_ignored_mismatch() {
        let mut redirect_server = mockito::Server::new_async().await;
        redirect_server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(301)
            .with_header("Location", &redirect_server.url())
            .create_async()
            .await;

        let result = check_http(
            &server.url(),
            &[WildMatch::new("http://is-mismatched.com/*")],
        )
        .await;

        assert_eq!(
            result,
            LinkCheckResult::Warning(format!(
                "Request was redirected to {}/",
                &redirect_server.url()
            ))
        );
    }

    #[tokio::test]
    async fn check_http_is_redirection_failure() {
        let mut redirect_server = mockito::Server::new_async().await;
        redirect_server
            .mock("GET", "/")
            .with_status(403)
            .create_async()
            .await;
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(301)
            .with_header("Location", &redirect_server.url())
            .create_async()
            .await;

        let result = check_http(&server.url(), &[]).await;

        assert_eq!(
            result,
            LinkCheckResult::Failed("403 - Forbidden".to_string())
        );
    }

    #[tokio::test]
    async fn check_http_with_fragment_no_warning() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/page")
            .with_status(200)
            .create_async()
            .await;

        // The URL with a fragment should not produce a redirect warning
        // because the fragment is not sent to the server
        let url_with_fragment = format!("{}/page#anchor", server.url());
        let result = check_http(&url_with_fragment, &[]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_with_fragment_real_redirect_warns() {
        let mut redirect_server = mockito::Server::new_async().await;
        redirect_server
            .mock("GET", "/other-page")
            .with_status(200)
            .create_async()
            .await;

        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/page")
            .with_status(301)
            .with_header("Location", &format!("{}/other-page", redirect_server.url()))
            .create_async()
            .await;

        // A real redirect to a different page should still produce a warning
        // even if the original URL had a fragment
        let url_with_fragment = format!("{}/page#anchor", server.url());
        let result = check_http(&url_with_fragment, &[]).await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(format!(
                "Request was redirected to {}/other-page",
                &redirect_server.url()
            ))
        );
    }
}
