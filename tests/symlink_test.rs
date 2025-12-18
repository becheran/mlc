#[cfg(test)]
use mlc::file_traversal;
use mlc::markup::{MarkupFile, MarkupType};
use mlc::Config;
use mlc::OptionalConfig;
use std::path::Path;

#[test]
fn test_symlink_dedupe() {
    let path = Path::new("./tests/test_files/symlink_test").to_path_buf();
    let config: Config = Config {
        directory: path,
        optional: OptionalConfig {
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    };
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config, &mut result);

    // Should find only 1 file (not 2) since symlink.md points to original.md
    assert_eq!(
        result.len(),
        1,
        "Expected to find only 1 file, but found {}: {:?}",
        result.len(),
        result
    );
}
