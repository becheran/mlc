#[cfg(test)]
mod helper;

use helper::benches_dir;
use mlc::markup::MarkupType;
use mlc::Config;
use mlc::OptionalConfig;
use mockito::ServerGuard;
use std::fs;
use std::path::PathBuf;

async fn setup_mock_servers() -> Vec<ServerGuard> {
    let mut servers = Vec::new();

    // Create multiple mock servers
    for _ in 0..8 {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("HEAD", "/")
            .with_status(200)
            .create_async()
            .await;
        server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;
        servers.push(server);
    }

    servers
}

fn replace_mock_urls(content: &str, servers: &[ServerGuard]) -> String {
    let mut result = content.to_string();
    for (i, server) in servers.iter().enumerate() {
        let placeholder = format!("MOCK_SERVER_URL_{}", i + 1);
        result = result.replace(&placeholder, &server.url());
    }
    result
}

fn test_files_dir() -> PathBuf {
    benches_dir()
        .parent()
        .unwrap()
        .join("tests")
        .join("test_files")
}

#[tokio::test]
async fn end_to_end_with_mock_servers() {
    // Set up mock servers
    let servers = setup_mock_servers().await;

    // Create temporary directory for test files with replaced URLs
    let temp_dir = std::env::temp_dir().join("mlc_test_mock_servers");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).unwrap();
    }
    fs::create_dir_all(&temp_dir).unwrap();
    fs::create_dir_all(temp_dir.join("deep")).unwrap();

    // Copy and replace URLs in test files
    let test_files = test_files_dir();

    // Reference links file
    let content = fs::read_to_string(test_files.join("reference_links.md")).unwrap();
    let updated_content = replace_mock_urls(&content, &servers);
    fs::write(temp_dir.join("reference_links.md"), updated_content).unwrap();

    // Many links file
    let content = fs::read_to_string(test_files.join("many_links.md")).unwrap();
    let updated_content = replace_mock_urls(&content, &servers);
    fs::write(temp_dir.join("many_links.md"), updated_content).unwrap();

    // Repeat links file
    let content = fs::read_to_string(test_files.join("repeat_links.md")).unwrap();
    let updated_content = replace_mock_urls(&content, &servers);
    fs::write(temp_dir.join("repeat_links.md"), updated_content).unwrap();

    // Deep directory file
    let content = fs::read_to_string(test_files.join("deep/index.md")).unwrap();
    fs::write(temp_dir.join("deep/index.md"), content).unwrap();

    // Run mlc with the temporary directory
    let config = Config {
        directory: temp_dir.clone(),
        optional: OptionalConfig {
            debug: Some(true),
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: None,
            match_file_extension: None,
            throttle: None,
            ignore_links: Some(vec![
                // Only ignore non-http links that are expected to be unsupported
                "mailto://*".to_string(),
                "another://*".to_string(),
            ]),
            ignore_path: None,
            root_dir: None,
            gitignore: None,
            gituntracked: None,
            csv_file: None,
            files: None,
            http_headers: None,
            disable_raw_link_check: None,
        },
    };

    // Run the link checker - should succeed because all mock servers return 200
    if let Err(e) = mlc::run(&config).await {
        panic!("Test failed with mock servers. {:?}", e);
    }

    // Clean up
    fs::remove_dir_all(&temp_dir).unwrap();
}

#[tokio::test]
async fn end_to_end_with_mock_server_failure() {
    // Set up a mock server that returns 404
    let mut server = mockito::Server::new_async().await;
    server
        .mock("HEAD", "/")
        .with_status(404)
        .create_async()
        .await;
    server
        .mock("GET", "/")
        .with_status(404)
        .create_async()
        .await;

    // Create temporary directory for test files
    let temp_dir = std::env::temp_dir().join("mlc_test_mock_failure");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).unwrap();
    }
    fs::create_dir_all(&temp_dir).unwrap();

    // Create a simple test file with a broken link
    let content = format!("[Broken Link]({})", server.url());
    fs::write(temp_dir.join("broken.md"), content).unwrap();

    let config = Config {
        directory: temp_dir.clone(),
        optional: OptionalConfig {
            debug: Some(true),
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: None,
            match_file_extension: None,
            throttle: None,
            ignore_links: None,
            ignore_path: None,
            root_dir: None,
            gitignore: None,
            gituntracked: None,
            csv_file: None,
            files: None,
            http_headers: None,
            disable_raw_link_check: None,
        },
    };

    // Run the link checker - should fail because server returns 404
    if mlc::run(&config).await.is_ok() {
        panic!("Test should have failed due to 404 response from mock server");
    }

    // Clean up
    fs::remove_dir_all(&temp_dir).unwrap();
}

#[tokio::test]
async fn end_to_end_with_mock_server_redirect() {
    // Set up redirect and target mock servers
    let mut target_server = mockito::Server::new_async().await;
    target_server
        .mock("HEAD", "/")
        .with_status(200)
        .create_async()
        .await;
    target_server
        .mock("GET", "/")
        .with_status(200)
        .create_async()
        .await;

    let mut redirect_server = mockito::Server::new_async().await;
    redirect_server
        .mock("HEAD", "/")
        .with_status(301)
        .with_header("Location", &target_server.url())
        .create_async()
        .await;
    redirect_server
        .mock("GET", "/")
        .with_status(301)
        .with_header("Location", &target_server.url())
        .create_async()
        .await;

    // Create temporary directory for test files
    let temp_dir = std::env::temp_dir().join("mlc_test_mock_redirect");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).unwrap();
    }
    fs::create_dir_all(&temp_dir).unwrap();

    // Create a test file with a redirect
    let content = format!("[Redirect Link]({})", redirect_server.url());
    fs::write(temp_dir.join("redirect.md"), content).unwrap();

    let config = Config {
        directory: temp_dir.clone(),
        optional: OptionalConfig {
            debug: Some(true),
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: None,
            match_file_extension: None,
            throttle: None,
            ignore_links: None,
            ignore_path: None,
            root_dir: None,
            gitignore: None,
            gituntracked: None,
            csv_file: None,
            files: None,
            http_headers: None,
            disable_raw_link_check: None,
        },
    };

    // Run the link checker - should succeed but with warnings
    // The run should succeed even with redirect warnings
    let result = mlc::run(&config).await;
    assert!(
        result.is_ok(),
        "Test should succeed even with redirect warnings"
    );

    // Clean up
    fs::remove_dir_all(&temp_dir).unwrap();
}
