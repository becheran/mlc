use crate::Config;
use crate::link_validator::LinkCheckResult;
use log::{debug, info};
use std::ffi::OsStr;
use std::fs;
use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR, Path, PathBuf};

pub fn check_filesystem(target: &str, config: &Config) -> LinkCheckResult {
    let target = Path::new(target);
    debug!("Absolute target path {target:?}");
    if target.exists() {
        return LinkCheckResult::Ok;
    }
    if config.optional.match_file_extension.unwrap_or_default() || target.extension().is_some() {
        return LinkCheckResult::Failed("Target filename not found.".to_string());
    }

    // Check if file exists ignoring the file extension
    let Some(target_file_name) = target.file_name() else {
        return LinkCheckResult::Failed("Target path not found.".to_string());
    };
    let Some(target_parent) = target.parent() else {
        return LinkCheckResult::Failed("Target parent not found.".to_string());
    };
    debug!("Check if file ignoring the extension exists.");
    if file_exists_ignoring_extension(target_parent, target_file_name) {
        LinkCheckResult::Ok
    } else {
        LinkCheckResult::Failed("Target not found.".to_string())
    }
}

/// Is there a file in `dir` which has the name `file_name` when its extension is removed?
fn file_exists_ignoring_extension(dir: &Path, file_name: &OsStr) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    // `DirEntry::file_type` does not follow symlinks, so links to directories count as files
    entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|file_type| !file_type.is_dir()))
        .find(|entry| Path::new(&entry.file_name()).file_stem() == Some(file_name))
        .inspect(|entry| info!("Found file {:?}", entry.path()))
        .is_some()
}

pub fn resolve_target_link(
    source: &str,
    canonical_source: Option<&Path>,
    target: &str,
    config: &Config,
) -> String {
    let mut normalized_link = target.replace(['/', '\\'], MAIN_SEPARATOR_STR);
    if let Some(idx) = normalized_link.find('#') {
        info!(
            "Strip everything after #. The chapter part '{}' is not checked.",
            &normalized_link[idx..]
        );
        normalized_link.truncate(idx);
    }
    let mut fs_link_target = PathBuf::from(&normalized_link);
    if let (Some(root_dir), Some(relative_link)) = (
        &config.optional.root_dir,
        normalized_link.strip_prefix(MAIN_SEPARATOR),
    ) {
        let new_root = fs::canonicalize(root_dir).unwrap_or_else(|e| {
            panic!(
                "Root path could not be converted to an absolute path. Does the directory exit? {e}"
            )
        });
        fs_link_target = new_root.join(relative_link);
    }

    debug!("Check file system link target {target:?}");
    let abs_path = absolute_target_path(source, canonical_source, &fs_link_target);
    let abs_path = abs_path.to_str().expect("Could not resolve target path");
    // Remove verbatim path identifier which causes trouble on windows when using ../../ in paths
    abs_path
        .strip_prefix(r"\\?\")
        .unwrap_or(abs_path)
        .to_string()
}

fn absolute_target_path(source: &str, canonical_source: Option<&Path>, target: &Path) -> PathBuf {
    if !target.is_relative() {
        return target.to_path_buf();
    }
    let abs_source = match canonical_source {
        Some(path) => path.to_path_buf(),
        None => fs::canonicalize(source).expect("Expected path to exist."),
    };
    let parent = abs_source
        .parent()
        .unwrap_or_else(|| Path::new(MAIN_SEPARATOR_STR));
    let target = target
        .strip_prefix(format!(".{MAIN_SEPARATOR}"))
        .unwrap_or(target);
    parent.join(target)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn remove_dot() {
        let source = Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("benches")
            .join("benchmark");
        let target = Path::new("./script_and_comments.md");

        let path = absolute_target_path(source.to_str().unwrap(), None, target);

        let path_str = path.to_str().unwrap().to_string();
        println!("{path_str:?}");
        assert_eq!(path_str.matches('.').count(), 1);
    }
}
