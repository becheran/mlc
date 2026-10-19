use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// A scratch directory with one markdown file with one valid, one broken and one web link.
/// The binary is started in it, so the `.mlc.toml` of the repository is not picked up.
struct Workspace(PathBuf);

impl Workspace {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("mlc_cli_output_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("a.md"),
            "# T\n[ok](./a.md)\n[broken](./missing.md)\n[web](https://example.invalid/x)\n",
        )
        .unwrap();
        Workspace(dir)
    }

    fn mlc(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_mlc"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn default_output_has_banner_and_all_links() {
    let ws = Workspace::new("default");
    let out = ws.mlc(&["--offline", "."]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout.contains("markup link checker - mlc"));
    assert!(stdout.contains("OK"));
    assert!(stdout.contains("The following links could not be resolved:"));
}

#[test]
fn quiet_prints_only_problems_and_summary() {
    let ws = Workspace::new("quiet");
    let out = ws.mlc(&["--offline", "--quiet", "."]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1));
    assert!(!stdout.contains("markup link checker - mlc"));
    assert!(!stdout.contains("[ OK ]"));
    assert!(!stdout.contains("[Skip]"));
    assert!(!stdout.contains("could not be resolved"));
    assert!(stdout.contains("Errors   1"));
    assert!(stderr.contains("missing.md"));
}

#[test]
fn quiet_without_errors_exits_with_zero() {
    let ws = Workspace::new("quiet_ok");
    fs::write(ws.0.join("a.md"), "# T\n[ok](./a.md)\n").unwrap();
    let out = ws.mlc(&["--offline", "--quiet", "."]);
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("Errors   0"));
    assert!(out.stderr.is_empty());
}

#[test]
fn json_stdout_is_a_single_document() {
    let ws = Workspace::new("json");
    let out = ws.mlc(&["--offline", "--format", "json", "."]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["summary"]["total"], 3);
    assert_eq!(json["summary"]["ok"], 1);
    assert_eq!(json["summary"]["skipped"], 1);
    assert_eq!(json["summary"]["warnings"], 0);
    assert_eq!(json["summary"]["errors"], 1);

    let issues = json["issues"].as_array().unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["severity"], "error");
    assert_eq!(issues[0]["line"], 3);
    assert_eq!(issues[0]["column"], 1);
    assert_eq!(issues[0]["target"], "./missing.md");
    assert_eq!(issues[0]["message"], "Target filename not found.");
    assert!(issues[0]["source"].as_str().unwrap().ends_with("a.md"));
}

#[test]
fn json_keeps_stdout_clean_in_debug_mode() {
    let ws = Workspace::new("json_debug");
    let out = ws.mlc(&["--offline", "--format", "json", "--debug", "."]);
    serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap();
    assert!(!out.stderr.is_empty());
}

#[test]
fn format_can_be_set_in_config_file() {
    let ws = Workspace::new("json_config");
    fs::write(
        ws.0.join(".mlc.toml"),
        "offline = true\nformat = \"json\"\n",
    )
    .unwrap();
    let out = ws.mlc(&["."]);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["summary"]["errors"], 1);
}

#[test]
fn unknown_format_is_rejected() {
    let ws = Workspace::new("bad_format");
    let out = ws.mlc(&["--format", "xml", "."]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn invalid_throttle_is_a_usage_error() {
    let ws = Workspace::new("bad_throttle");
    let out = ws.mlc(&["--offline", "--throttle", "fast", "."]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--throttle"));
}

#[test]
fn invalid_config_file_is_reported_without_panic() {
    let ws = Workspace::new("bad_config");
    fs::write(
        ws.0.join(".mlc.toml"),
        "offline = [
",
    )
    .unwrap();
    let out = ws.mlc(&["."]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr.contains("Invalid TOML file"));
    assert!(!stderr.contains("panicked"));
}

#[test]
fn unwritable_csv_file_is_reported_without_panic() {
    let ws = Workspace::new("bad_csv");
    let out = ws.mlc(&["--offline", "--csv", "missing_dir/report.csv", "."]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr.contains("Failed to write CSV file"));
    assert!(!stderr.contains("panicked"));
}

#[test]
fn gitignore_set_to_false_in_config_file_does_not_enable_it() {
    let ws = Workspace::new("gitignore_false");
    fs::write(
        ws.0.join(".mlc.toml"),
        "offline = true
gitignore = false
",
    )
    .unwrap();
    let out = ws.mlc(&["--format", "json", "."]);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    // The broken link is found, so the file was not skipped because of git
    assert_eq!(json["summary"]["errors"], 1);
    assert_eq!(json["summary"]["skipped"], 1);
}
