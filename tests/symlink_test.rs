use mlc::file_traversal;
use mlc::markup::{MarkupFile, MarkupType};
use mlc::Config;
use mlc::OptionalConfig;
use std::fs;
use std::path::Path;

#[cfg(unix)]
fn create_symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(windows)]
fn create_symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(original, link)
}

#[test]
fn test_symlink_dedupe() {
    // Create the symlink at runtime so the test does not depend on how git checked it out
    // (e.g. core.symlinks=false on Windows turns symlinks into plain files).
    let dir = std::env::temp_dir().join(format!("mlc_symlink_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let original = dir.join("original.md");
    fs::write(&original, "[link](original.md)\n").unwrap();

    if let Err(e) = create_symlink(&original, &dir.join("symlink.md")) {
        eprintln!("Skipping test_symlink_dedupe: cannot create symlinks here ({e})");
        let _ = fs::remove_dir_all(&dir);
        return;
    }

    let config: Config = Config {
        directory: dir.clone(),
        optional: OptionalConfig {
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    };
    let mut result: Vec<MarkupFile> = Vec::new();

    file_traversal::find(&config, &mut result);
    let _ = fs::remove_dir_all(&dir);

    // Should find only 1 file (not 2) since symlink.md points to original.md
    assert_eq!(
        result.len(),
        1,
        "Expected to find only 1 file, but found {}: {:?}",
        result.len(),
        result
    );
}
