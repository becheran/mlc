use mlc::markup::MarkupType;
use mlc::Config;
use mlc::OptionalConfig;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Self {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "mlc_test_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        path.push(unique);
        fs::create_dir_all(&path).expect("failed to create temp dir");
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_git(repo: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(repo)
        .args(args)
        .status()
        .expect("failed to run git");
    assert!(status.success(), "git command failed: git {:?}", args);
}

#[tokio::test]
async fn gitignore_is_recursive_nested_gitignore_is_respected() {
    if !git_available() {
        panic!("Failing test: git executable must be available");
    }

    let repo = TempDir::new("gitignore_recursive");

    // Create nested structure
    let docs_dir = repo.path.join("docs");
    fs::create_dir_all(&docs_dir).expect("failed to create docs dir");

    // Nested .gitignore ignores only ignored.md (not configured in root .gitignore)
    fs::write(docs_dir.join(".gitignore"), "ignored.md\n")
        .expect("failed to write nested .gitignore");

    // Tracked files (should be checked)
    fs::write(docs_dir.join("ok_target.md"), "# ok\n").expect("failed to write ok_target.md");
    fs::write(docs_dir.join("checked.md"), "[ok](./ok_target.md)\n")
        .expect("failed to write checked.md");

    // Ignored file contains a broken link; if this file is (incorrectly) checked, mlc should fail.
    fs::write(docs_dir.join("ignored.md"), "[broken](./missing.md)\n")
        .expect("failed to write ignored.md");

    // Initialize git repo and commit tracked files.
    run_git(&repo.path, &["init"]);
    run_git(&repo.path, &["config", "user.email", "test@example.com"]);
    run_git(&repo.path, &["config", "user.name", "mlc test"]);
    run_git(
        &repo.path,
        &[
            "add",
            "docs/.gitignore",
            "docs/ok_target.md",
            "docs/checked.md",
        ],
    );
    run_git(&repo.path, &["commit", "-m", "test fixtures"]);

    let config = Config {
        directory: repo.path.clone(),
        optional: OptionalConfig {
            debug: None,
            do_not_warn_for_redirect_to: None,
            markup_types: Some(vec![MarkupType::Markdown]),
            offline: Some(true),
            match_file_extension: None,
            ignore_links: None,
            ignore_path: None,
            root_dir: None,
            gitignore: Some(true),
            gituntracked: None,
            csv_file: None,
            throttle: None,
            files: None,
            http_headers: None,
        },
    };

    let result = mlc::run(&config).await;

    assert!(
        result.is_ok(),
        "Expected ok because ignored.md should be ignored by nested .gitignore"
    );
}
