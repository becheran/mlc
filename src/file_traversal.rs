extern crate walkdir;

use walkdir::WalkDir;
use crate::Config;
use crate::markup::MarkupType;

pub fn find(config: &Config, result: &mut Vec<String>) {
    let root = &config.folder;
    let markup_types = &config.markup_types;

    info!("Search for files of markup types '{:?}' in directory '{}'", markup_types, root);

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();
        if has_file_extension(&f_name, &markup_types) {
            let file = entry.path().to_string_lossy().to_string();
            debug!("Found file: {}", file);
            result.push(file);
        }
    }
}

fn has_file_extension(file: &str, markup_types: &[MarkupType]) -> bool {
    let file_low = file.to_lowercase();
    //TODO speedup!
    for t in markup_types {
        let extensions = t.file_extensions();
        for ext in extensions {
            let ext_low = ext.to_lowercase();
            if file_low.ends_with(&ext_low) {
                return true;
            }
        }
    }

    false
}