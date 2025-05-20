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
            offline: None,
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
        },
    };
    if let Err(e) = mlc::run(&config).await {
        panic!("Test failed. {:?}", e);
    }
}

#[tokio::test]
async fn end_to_end_different_root() {
    let test_files = benches_dir().join("different_root");
    let csv_output = std::env::temp_dir().join("mlc_test_output.csv");
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
        },
    };
    if let Err(e) = mlc::run(&config).await {
        panic!("Test with custom root failed. {:?}", e);
    } else {
        // Check if the CSV file was created, but is empty except for the header
        let content = fs::read_to_string(csv_output).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "source,line,column,target");
    }
}

#[tokio::test]
async fn end_to_end_write_csv_file() {
    let csv_output = std::env::temp_dir().join("mlc_test_output.csv");
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
        },
    };
    if let Err(_) = mlc::run(&config).await {
        let content = fs::read_to_string(csv_output).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "source,line,column,target");
        for i in 1..lines.len() {
            assert_eq!(
                lines[i],
                &format!(
                    "benches{MAIN_SEPARATOR}benchmark/markdown/ignore_me.md,{i},1,broken_Link",
                )
            );
        }
    } else {
        panic!("Should have detected errors");
    }
}
