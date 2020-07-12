use crate::link_validator::LinkCheckResult;

use reqwest::Client;
use reqwest::Method;
use reqwest::Request;
use reqwest::StatusCode;

pub async fn check_http(target: &str) -> LinkCheckResult {
    debug!("Check http link target {:?}", target);
    let url = reqwest::Url::parse(&target).expect("URL of unknown type");

    match http_request(&url).await {
        Ok(response) => response,
        Err(error_msg) => LinkCheckResult::Failed(format!("Http(s) request failed. {}", error_msg)),
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn check_http_is_available() {
        let result = check_http("http://gitlab.com/becheran/mlc").await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_request_with_hash() {
        let result = check_http("http://gitlab.com/becheran/mlc#bla").await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_wrong_http_request() {
        let result = check_http("https://doesNotExist.me/even/less/likelly").await;
        assert!(result != LinkCheckResult::Ok);
    }
}
