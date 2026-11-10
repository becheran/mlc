//! Asks the `git` binary which markup files are ignored or untracked

use crate::Config;
use crate::markup::MarkupType;
use log::debug;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MARKUP_TYPES: [MarkupType; 2] = [MarkupType::Markdown, MarkupType::Html];

/// Markup files which are ignored by git. `None` if `git` could not be asked.
pub fn ignored_files(config: &Config) -> Option<HashSet<PathBuf>> {
    ls_files(config, &["--ignored", "--others", "--exclude-standard"])
}

/// Markup files which are not tracked by git. `None` if `git` could not be asked.
pub fn untracked_files(config: &Config) -> Option<HashSet<PathBuf>> {
    ls_files(config, &["--others", "--exclude-standard"])
}

fn repo_root(scan_root: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(scan_root)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if !output.status.success() {
        debug!(
            "git rev-parse failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    let root = String::from_utf8(output.stdout).ok()?;
    let root = root.trim();
    (!root.is_empty()).then(|| PathBuf::from(root))
}

fn scan_root_dir(config: &Config) -> &Path {
    let p = config.directory.as_path();
    if p.is_file() {
        p.parent().unwrap_or_else(|| Path::new("."))
    } else {
        p
    }
}

fn ls_files(config: &Config, args: &[&str]) -> Option<HashSet<PathBuf>> {
    let scan_root = scan_root_dir(config);
    let repo_root = repo_root(scan_root)?;

    // Limit ls-files to the scanned subtree so nested .gitignore files are respected
    // and we don't accidentally base results on the caller's current working directory.
    let output = Command::new("git")
        .arg("-C")
        .arg(&repo_root)
        .arg("ls-files")
        .args(args)
        .args(["--", "."])
        .current_dir(scan_root)
        .output()
        .ok()?;

    if !output.status.success() {
        eprintln!(
            "git ls-files command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return None;
    }

    let files = String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .filter(|line| MarkupType::from_file_name(line, &MARKUP_TYPES).is_some())
        .filter_map(|line| fs::canonicalize(repo_root.join(line.trim())).ok())
        .collect();
    Some(files)
}
