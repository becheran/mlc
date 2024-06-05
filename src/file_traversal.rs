extern crate walkdir;

use crate::markup::{MarkupFile, MarkupType};
use crate::Config;
use std::fs;
use walkdir::WalkDir;

pub fn find(config: &Config, result: &mut Vec<MarkupFile>) {
    let root = &config.directory;
    let markup_types = match &config.optional.markup_types {
        Some(t) => t,
        None => panic!("Bug! markup_types must be set"),
    };

    info!(
        "Search for files of markup types '{:?}' in directory '{:?}'",
        markup_types, root
    );

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
                            info!("Ignore directory: '{:?}'", f);
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

        if let Some(markup_type) = markup_type(&f_name, &markup_types) {
            let path = entry.path();

            let abs_path = match fs::canonicalize(path) {
                Ok(abs_path) => abs_path,
                Err(e) => {
                    warn!("Path '{:?}' not able to canonicalize path. '{e}'", path);
                    continue;
                }
            };

            let ignore = match &config.optional.ignore_path {
                Some(p) => p.iter().any(|ignore_path| ignore_path == &abs_path),
                None => false,
            };
            if ignore {
                debug!("Ignore file {f_name}, because it is in the ignore path list.");
            } else {
                let file = MarkupFile {
                    markup_type,
                    path: path.to_string_lossy().to_string(),
                };
                debug!("Found file: {:?}.", file);
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
