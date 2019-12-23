extern crate walkdir;

use walkdir::WalkDir;
use crate::Config;
use crate::markup::{MarkupType, MarkupFile};

pub fn find(config: &Config, result: &mut Vec<MarkupFile>) {
    let root = &config.folder;
    let markup_types = &config.markup_types;

    info!("Search for files of markup types '{:?}' in directory '{}'", markup_types, root);

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir()) {
        let f_name = entry.file_name().to_string_lossy();

        if let Some(markup_type) = markup_type(&f_name, &markup_types) {
            let path = entry.path().to_string_lossy().to_string();
            let file = MarkupFile {
                markup_type,
                path,
            };
            debug!("Found file: {:?}", file);
            result.push(file);
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
                return Some(markup_type.clone());
            }
        }
    }

    None
}