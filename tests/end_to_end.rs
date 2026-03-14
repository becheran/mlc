#[cfg(test)]
mod helper;

use helper::benches_dir;
use mlc::markup::MarkupType;
use mlc::Config;
use mlc::OptionalConfig;
use std::fs;
use std::path::MAIN_SEPARATOR;

#[tokio::test]
async fn end_to_end() {
    let config = Config {
        directory: benches_dir().join("benchmark"),
        optional: OptionalConfig {
            debug: None,
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: Some(true), // Set to offline mode to avoid external network calls
            match_file_extension: None,
            throttle: None,
            ignore_links: Some(vec!["./doc/broken-local-link.doc".to_string()]),
            ignore_path: Some(vec![
                fs::canonicalize("benches/benchmark/markdown/ignore_me.md").unwrap(),
                fs::canonicalize("./benches/benchmark/markdown/ignore_me_dir").unwrap(),
            ]),
            root_dir: None,
            gitignore: None,
            gituntracked: None,
            csv_file: None,
            disable_raw_link_check: None,
            files: None,
            http_headers: None,
        },
    };
    if let Err(e) = mlc::run(&config).await {
        panic!("Test failed. {:?}", e);
    }
}

#[tokio::test]
async fn end_to_end_different_root() {
    let test_files = benches_dir().join("different_root");
    let csv_output = std::env::temp_dir().join("mlc_test_different_root.csv");
    let config = Config {
        directory: test_files.clone(),
        optional: OptionalConfig {
            debug: Some(true),
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: None,
            match_file_extension: None,
            ignore_links: None,
            ignore_path: None,
            throttle: None,
            root_dir: Some(test_files),
            gitignore: None,
            gituntracked: None,
            csv_file: Some(csv_output.clone()),
            disable_raw_link_check: None,
            files: None,
            http_headers: None,
        },
    };
    if let Err(e) = mlc::run(&config).await {
        panic!("Test with custom root failed. {:?}", e);
    } else {
        // Check if the CSV file was created, but is empty except for the header
        let content = fs::read_to_string(csv_output).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "source,line,column,target,severity");
    }
}

#[tokio::test]
async fn end_to_end_write_csv_file() {
    let csv_output = std::env::temp_dir().join("mlc_test_write_csv.csv");
    let config = Config {
        directory: benches_dir().join("benchmark/markdown/ignore_me.md"),
        optional: OptionalConfig {
            debug: None,
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
            csv_file: Some(csv_output.clone()),
            disable_raw_link_check: None,
            files: None,
            http_headers: None,
        },
    };
    if (mlc::run(&config).await).is_err() {
        let content = fs::read_to_string(csv_output).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "source,line,column,target,severity");
        for (i, line) in lines.iter().enumerate().skip(1) {
            assert_eq!(
                line,
                &format!(
                    "benches{MAIN_SEPARATOR}benchmark/markdown/ignore_me.md,{i},1,broken_Link,ERR",
                )
            );
        }
    } else {
        panic!("Should have detected errors");
    }
}

#[tokio::test]
async fn end_to_end_csv_include_warnings() {
    let csv_output = std::env::temp_dir().join("mlc_test_csv_warnings.csv");
    let config = Config {
        directory: benches_dir().join("benchmark/markdown/ref_links.md"),
        optional: OptionalConfig {
            debug: None,
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: Some(true), // Use offline mode to avoid actual HTTP calls
            match_file_extension: None,
            throttle: None,
            ignore_links: None,
            ignore_path: None,
            root_dir: None,
            gitignore: None,
            gituntracked: None,
            csv_file: Some(csv_output.clone()),
            files: None,
            http_headers: None,
            disable_raw_link_check: None,
        },
    };
    // Run the check - should succeed because we're offline
    let result = mlc::run(&config).await;

    // Check that CSV was created
    assert!(csv_output.exists(), "CSV file should exist");

    let content = fs::read_to_string(&csv_output).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Should have header and warning entries
    assert!(
        lines.len() > 1,
        "CSV should have header and warning entries"
    );
    assert_eq!(lines[0], "source,line,column,target,severity");

    // Verify that warning entries are present - the ref_links.md file has several broken markdown references
    // Check that all lines after header have the expected CSV format with severity column
    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split(',').collect();
        assert_eq!(
            parts.len(),
            5,
            "Each CSV line should have 5 columns including severity"
        );
        assert!(
            parts[0].contains("ref_links.md"),
            "Source should be ref_links.md"
        );
        assert_eq!(parts[4], "WARN", "Severity should be WARN for warnings");
    }

    // Verify specific warnings are captured (broken markdown references)
    assert!(
        content.contains(",WARN"),
        "CSV should contain WARN severity"
    );

    // Clean up
    let _ = fs::remove_file(csv_output);

    // Also verify the test would pass
    assert!(result.is_ok(), "Should succeed with warnings only");
}

#[tokio::test]
async fn end_to_end_code_block_links_enabled() {
    // Test that raw links in code blocks are checked by default
    let test_file = "tests/test_files/code_block_links.md";
    let config = Config {
        directory: test_file.into(),
        optional: OptionalConfig {
            debug: None,
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: Some(true), // Offline mode to avoid actual HTTP calls
            match_file_extension: None,
            throttle: None,
            ignore_links: None,
            ignore_path: None,
            root_dir: None,
            gitignore: None,
            gituntracked: None,
            csv_file: None,
            disable_raw_link_check: None, // Default: enabled (checks raw links in code blocks)
            files: None,
            http_headers: None,
        },
    };

    // Run the check - should succeed in offline mode (links are skipped but counted)
    let result = mlc::run(&config).await;
    assert!(
        result.is_ok(),
        "Should succeed in offline mode: {:?}",
        result
    );
}

#[tokio::test]
async fn end_to_end_code_block_links_disabled() {
    // Test that raw links in code blocks can be disabled
    let test_file = "tests/test_files/code_block_links.md";
    let config = Config {
        directory: test_file.into(),
        optional: OptionalConfig {
            debug: None,
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: Some(true), // Offline mode to avoid actual HTTP calls
            match_file_extension: None,
            throttle: None,
            ignore_links: None,
            ignore_path: None,
            root_dir: None,
            gitignore: None,
            gituntracked: None,
            csv_file: None,
            disable_raw_link_check: Some(true), // Disable raw link checking
            files: None,
            http_headers: None,
        },
    };

    // Run the check - should succeed
    let result = mlc::run(&config).await;
    assert!(
        result.is_ok(),
        "Should succeed with raw link checking disabled: {:?}",
        result
    );
}
