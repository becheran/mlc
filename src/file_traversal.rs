use crate::Config;
use crate::markup::MarkupFile;
use crate::markup::MarkupType;
use log::{debug, info, warn};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn find(config: &Config, result: &mut Vec<MarkupFile>) {
    let Some(markup_types) = &config.optional.markup_types else {
        panic!("Bug! markup_types must be set");
    };
    let mut seen_paths: HashSet<PathBuf> = HashSet::new();

    // Adds the file unless it is ignored or was already added via another path
    let mut add_file = |path: &Path, f_name: &str, markup_type: MarkupType| {
        let abs_path = match fs::canonicalize(path) {
            Ok(abs_path) => abs_path,
            Err(e) => {
                warn!("Path '{path:?}' not able to canonicalize path. '{e}'");
                return;
            }
        };

        let is_ignored = (config.optional.ignore_path.iter().flatten()).any(|p| *p == abs_path);
        if is_ignored {
            debug!("Ignore file {f_name}, because it is in the ignore path list.");
        } else if seen_paths.contains(&abs_path) {
            debug!("Skip file {f_name}, already checked via canonical path: {abs_path:?}");
        } else {
            seen_paths.insert(abs_path);
            let file = MarkupFile {
                markup_type,
                path: path.to_string_lossy().to_string(),
            };
            debug!("Found file: {file:?}.");
            result.push(file);
        }
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
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();

            debug!("Check file: '{f_name}'");

            match MarkupType::from_file_name(&f_name, markup_types) {
                Some(markup_type) => add_file(file_path, &f_name, markup_type),
                None => warn!("File '{f_name}' does not match any supported markup type."),
            }
        }
        return;
    }

    // Otherwise, use directory traversal
    let root = &config.directory;
    info!("Search for files of markup types '{markup_types:?}' in directory '{root:?}'");

    // Canonicalize the ignored directories once instead of for every visited entry
    let ignore_dirs: Vec<(&PathBuf, PathBuf)> = config
        .optional
        .ignore_path
        .iter()
        .flatten()
        .filter(|f| f.is_dir())
        .map(|f| (f, fs::canonicalize(f).unwrap_or_default()))
        .collect();

    let entries = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if !e.file_type().is_dir() || ignore_dirs.is_empty() {
                return true;
            }
            let canonical = e.path().canonicalize().unwrap_or_default();
            !ignore_dirs.iter().any(|(f, canonical_ignore)| {
                let ignore = canonical.starts_with(canonical_ignore);
                if ignore {
                    info!("Ignore directory: '{f:?}'");
                }
                ignore
            })
        })
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir());

    for entry in entries {
        let f_name = entry.file_name().to_string_lossy();
        debug!("Check file: '{f_name}'");

        if let Some(markup_type) = MarkupType::from_file_name(&f_name, markup_types) {
            add_file(entry.path(), &f_name, markup_type);
        }
    }
}
