use crate::Config;
use crate::link_validator::LinkCheckResult;
use async_std::path::Path;
use std::path::MAIN_SEPARATOR;
use async_std::path::PathBuf;

pub async fn check_filesystem(source: &str, target: &str, config: &Config) -> LinkCheckResult {
    let mut normalized_link = target
        .replace('/', &MAIN_SEPARATOR.to_string())
        .replace('\\', &MAIN_SEPARATOR.to_string());
    if let Some(idx) = normalized_link.find('#') {
        warn!(
            "Strip everything after #. The chapter part ´{}´ is not checked.",
            &normalized_link[idx..]
        );
        normalized_link = normalized_link[..idx].to_string();
    }
    let mut fs_link_target = Path::new(&normalized_link).to_path_buf();
    if normalized_link.starts_with(MAIN_SEPARATOR) && config.root_dir.is_some() {
        match async_std::fs::canonicalize(&config.root_dir.as_ref().unwrap()).await {
            Ok(new_root) => fs_link_target = new_root.join(Path::new(&normalized_link[1..])),
            Err(e) => panic!(
                "Root path could not be converted to an absolute path. Does the directory exit? {}",
                e
            ),
        }
    }

    debug!("Check file system link target {:?}", target);
    let target = absolute_target_path(source, &fs_link_target);
    debug!("Absolute target path {:?}", target);
    if target.exists().await {
        LinkCheckResult::Ok
    } else {
        LinkCheckResult::Failed("Target path not found.".to_string())
    }
}

fn absolute_target_path(source: &str, target: &PathBuf) -> PathBuf {
    if target.is_relative() {
        let parent = Path::new(source).parent().unwrap_or(Path::new("./"));
        parent.join(target)
    } else {
        target.to_owned()
    }
}
