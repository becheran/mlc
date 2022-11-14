use crate::link_validator::LinkCheckResult;

use futures::Future;
use reqwest::header::ACCEPT;
use reqwest::header::USER_AGENT;
use reqwest::Client;
use reqwest::Method;
use reqwest::redirect::Policy;
use reqwest::Request;
use reqwest::Response;
use reqwest::StatusCode;

pub async fn check_http(target: &str) -> LinkCheckResult {
    debug!("Check http link target {:?}", target);
    let url = reqwest::Url::parse(target).expect("URL of unknown type");

    match http_request(&url).await {
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

async fn http_request(url: &reqwest::Url) -> reqwest::Result<LinkCheckResult> {   
    async fn verify_redirection_status(url: &reqwest::Url, warning_message: String) -> reqwest::Result<LinkCheckResult> {
        let r = http_request_impl(url, true).await;
        match r {
            Ok(LinkCheckResult::Ok) => Ok(LinkCheckResult::Warning(warning_message)),
            Ok(r) => Ok(r),
            Err(r) => Err(r)
        }
    }
    
    match http_request_impl(url, false).await {
        Ok(LinkCheckResult::Warning(e)) => verify_redirection_status(url, e).await,
        Ok(r) => Ok(r),
        Err(e) => Err(e)
    }
}

async fn http_request_impl(url: &reqwest::Url, follow_redirections: bool) -> reqwest::Result<LinkCheckResult> {
    lazy_static! {
        static ref CLIENT: Client = Client::builder()
            .redirect(Policy::none())
            .build()
            .unwrap();
        static ref REDIRECTIONS_FOLLOWING_CLIENT: Client = Client::new();
    }

    fn status_to_string(status: StatusCode) -> String {
        format!(
            "{} - {}",
            status.as_str(),
            status.canonical_reason().unwrap_or("Unknown reason")
        )
    }

    fn execute(request: Request, follow_redirections: bool) -> impl Future<Output = Result<Response, reqwest::Error>> {
        if follow_redirections {
            return REDIRECTIONS_FOLLOWING_CLIENT.execute(request);
        }
        return CLIENT.execute(request);
    }

    let head_request = new_request(Method::HEAD, url);
    let get_request = new_request(Method::GET, url);

    let response = match execute(head_request, follow_redirections).await {
        Ok(r) => r,
        Err(e) => {
            println!("Head request error: {}. Retry with get-request.", e);
            execute(get_request, follow_redirections).await?
        }
    };

    let status = response.status();
    if status.is_success() {
        Ok(LinkCheckResult::Ok)
    } else if status.is_redirection() {
        Ok(LinkCheckResult::Warning(status_to_string(status)))
    } else {
        debug!("Got the status code {:?}. Retry with get-request.", status);
        let get_request = Request::new(Method::GET, url.clone());
        let response = execute(get_request, follow_redirections).await?;
        let status = response.status();
        if status.is_success() {
            Ok(LinkCheckResult::Ok)
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
        let result = check_http("https://gitlab.com/becheran/mlc").await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_is_redirection() {
        let result = check_http("http://gitlab.com/becheran/mlc").await;
        assert_eq!(result, LinkCheckResult::Warning("301 - Moved Permanently".to_string()));
    }

    #[tokio::test]
    async fn check_http_is_redirection_failure() {
        let result = check_http("http://github.com/fake-page").await;
        assert_eq!(result, LinkCheckResult::Warning("404 - Not Found".to_string()));
    }

    #[tokio::test]
    async fn check_https_crates_io_available() {
        let result = check_http("https://crates.io").await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_request_with_hash() {
        let result = check_http("https://gitlab.com/becheran/mlc#bla").await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_request_redirection_with_hash() {
        let result = check_http("http://gitlab.com/becheran/mlc#bla").await;
        assert_eq!(result, LinkCheckResult::Warning("301 - Moved Permanently".to_string()));
    }

    #[tokio::test]
    async fn check_wrong_http_request() {
        let result = check_http("https://doesNotExist.me/even/less/likelly").await;
        assert!(result != LinkCheckResult::Ok);
    }
}
