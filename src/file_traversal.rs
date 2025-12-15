extern crate walkdir;

use crate::markup::{MarkupFile, MarkupType};
use crate::Config;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn find(config: &Config, result: &mut Vec<MarkupFile>) {
    let mut seen_paths: HashSet<PathBuf> = HashSet::new();
    let markup_types = match &config.optional.markup_types {
        Some(t) => t,
        None => panic!("Bug! markup_types must be set"),
    };

    // If specific files are provided, process only those files
    if let Some(files) = &config.optional.files {
        info!("Checking specific files: {files:?}");

        for file_path in files {
            if !file_path.exists() {
                warn!("File path '{file_path:?}' does not exist.");
                continue;
            }

            if !file_path.is_file() {
                warn!("Path '{file_path:?}' is not a file.");
                continue;
            }

            let f_name = file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            debug!("Check file: '{f_name}'");

            if let Some(markup_type) = markup_type(&f_name, markup_types) {
                let abs_path = match fs::canonicalize(file_path) {
                    Ok(abs_path) => abs_path,
                    Err(e) => {
                        warn!("Path '{file_path:?}' not able to canonicalize path. '{e}'");
                        continue;
                    }
                };

                // Skip if we've already seen this canonical path
                if seen_paths.contains(&abs_path) {
                    debug!(
                        "Skip file {f_name}, already checked via canonical path: {:?}",
                        abs_path
                    );
                    continue;
                }

                let ignore = match &config.optional.ignore_path {
                    Some(p) => p.iter().any(|ignore_path| ignore_path == &abs_path),
                    None => false,
                };

                if ignore {
                    debug!("Ignore file {f_name}, because it is in the ignore path list.");
                } else {
                    seen_paths.insert(abs_path);
                    let file = MarkupFile {
                        markup_type,
                        path: file_path.to_string_lossy().to_string(),
                    };
                    debug!("Found file: {file:?}.");
                    result.push(file);
                }
            } else {
                warn!("File '{f_name}' does not match any supported markup type.");
            }
        }
        return;
    }

    // Otherwise, use directory traversal
    let root = &config.directory;
    info!("Search for files of markup types '{markup_types:?}' in directory '{root:?}'");

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            !(e.file_type().is_dir()
                && config.optional.ignore_path.as_ref().is_some_and(|x| {
                    x.iter().any(|f| {
                        let ignore = f.is_dir()
                            && e.path()
                                .canonicalize()
                                .unwrap_or_default()
                                .starts_with(fs::canonicalize(f).unwrap_or_default());
                        if ignore {
                            info!("Ignore directory: '{f:?}'");
                        }
                        ignore
                    })
                }))
        })
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let f_name = entry.file_name().to_string_lossy();
        debug!("Check file: '{f_name}'");

        if let Some(markup_type) = markup_type(&f_name, markup_types) {
            let path = entry.path();

            let abs_path = match fs::canonicalize(path) {
                Ok(abs_path) => abs_path,
                Err(e) => {
                    warn!("Path '{path:?}' not able to canonicalize path. '{e}'");
                    continue;
                }
            };

            // Skip if we've already seen this canonical path
            if seen_paths.contains(&abs_path) {
                debug!(
                    "Skip file {f_name}, already checked via canonical path: {:?}",
                    abs_path
                );
                continue;
            }

            let ignore = match &config.optional.ignore_path {
                Some(p) => p.iter().any(|ignore_path| ignore_path == &abs_path),
                None => false,
            };
            if ignore {
                debug!("Ignore file {f_name}, because it is in the ignore path list.");
            } else {
                seen_paths.insert(abs_path);
                let file = MarkupFile {
                    markup_type,
                    path: path.to_string_lossy().to_string(),
                };
                debug!("Found file: {file:?}.");
                result.push(file);
            }
        }
    }
}

fn markup_type(file: &str, markup_types: &[MarkupType]) -> Option<MarkupType> {
    let file_low = file.to_lowercase();
    for markup_type in markup_types {
        let extensions = markup_type.file_extensions();
        for ext in extensions {
            let mut ext_low = String::from(".");
            ext_low.push_str(&ext.to_lowercase());
            if file_low.ends_with(&ext_low) {
                return Some(*markup_type);
            }
        }
    }

    None
}
