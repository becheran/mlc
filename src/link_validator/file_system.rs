use crate::link_validator::LinkCheckResult;
use crate::Config;
use async_std::fs::canonicalize;
use async_std::path::Path;
use async_std::path::PathBuf;
use std::path::MAIN_SEPARATOR;
use walkdir::WalkDir;

pub async fn check_filesystem(target: &str, config: &Config) -> LinkCheckResult {
    let target = Path::new(target);
    debug!("Absolute target path {:?}", target);
    if target.exists().await {
        LinkCheckResult::Ok
    } else if !config.optional.match_file_extension.unwrap_or_default()
        && target.extension().is_none()
    {
        // Check if file exists ignoring the file extension
        let target_file_name = match target.file_name() {
            Some(s) => s,
            None => return LinkCheckResult::Failed("Target path not found.".to_string()),
        };
        let target_parent = match target.parent() {
            Some(s) => s,
            None => return LinkCheckResult::Failed("Target parent not found.".to_string()),
        };
        debug!("Check if file ignoring the extension exists.");
        if target_parent.exists().await {
            debug!(
                "Parent {:?} exists. Search dir for file ignoring the extension.",
                target_parent
            );
            for entry in WalkDir::new(target_parent)
                .follow_links(false)
                .max_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| !e.file_type().is_dir())
            {
                let mut file_on_system = entry.into_path();
                file_on_system.set_extension("");
                match file_on_system.file_name() {
                    Some(file_name) => {
                        if target_file_name == file_name {
                            info!("Found file {:?}", file_on_system);
                            return LinkCheckResult::Ok;
                        }
                    }
                    None => {
                        return LinkCheckResult::Failed("Target filename not found.".to_string())
                    }
                }
            }
            LinkCheckResult::Failed("Target not found.".to_string())
        } else {
            LinkCheckResult::Failed("Target not found.".to_string())
        }
    } else {
        LinkCheckResult::Failed("Target filename not found.".to_string())
    }
}

pub async fn resolve_target_link(source: &str, target: &str, config: &Config) -> String {
    let mut normalized_link = target
        .replace('/', &MAIN_SEPARATOR.to_string())
        .replace('\\', &MAIN_SEPARATOR.to_string());
    if let Some(idx) = normalized_link.find('#') {
        warn!(
            "Strip everything after #. The chapter part '{}' is not checked.",
            &normalized_link[idx..]
        );
        normalized_link = normalized_link[..idx].to_string();
    }
    let mut fs_link_target = Path::new(&normalized_link).to_path_buf();
    if normalized_link.starts_with(MAIN_SEPARATOR) && config.optional.root_dir.is_some() {
        match canonicalize(&config.optional.root_dir.as_ref().unwrap()).await {
            Ok(new_root) => fs_link_target = new_root.join(Path::new(&normalized_link[1..])),
            Err(e) => panic!(
                "Root path could not be converted to an absolute path. Does the directory exit? {}",
                e
            ),
        }
    }

    debug!("Check file system link target {:?}", target);
    let abs_path = absolute_target_path(source, &fs_link_target)
        .await
        .to_str()
        .expect("Could not resolve target path")
        .to_string();
    // Remove verbatim path identifier which causes trouble on windows when using ../../ in paths
    return abs_path
        .strip_prefix("\\\\?\\")
        .unwrap_or(&abs_path)
        .to_string();
}

async fn absolute_target_path(source: &str, target: &PathBuf) -> PathBuf {
    let abs_source = canonicalize(source).await.expect("Expected path to exist.");
    if target.is_relative() {
        let root = format!("{}", MAIN_SEPARATOR);
        let parent = abs_source.parent().unwrap_or_else(|| Path::new(&root));
        let new_target = match target.strip_prefix(format!(".{}", MAIN_SEPARATOR)) {
            Ok(t) => t,
            Err(_) => target,
        };
        parent.join(new_target)
    } else {
        target.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn remove_dot() {
        let source = Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("benches")
            .join("benchmark");
        let target = Path::new("./script_and_comments.md").to_path_buf();

        let path = absolute_target_path(source.to_str().unwrap(), &target).await;

        let path_str = path.to_str().unwrap().to_string();
        println!("{:?}", path_str);
        assert_eq!(path_str.matches('.').count(), 1);
    }
}
