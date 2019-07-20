extern crate walkdir;

use walkdir::WalkDir;

pub fn find(root: &str, file_extensions: &[String], result: &mut Vec<String>) {
    info!("Search for files with extension '{:?}' in directory '{}'", file_extensions, root);
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();
        if has_file_extension(&f_name, &file_extensions) {
            let file = entry.path().to_string_lossy().to_string();
            debug!("Found file: {}", file);
            result.push(file);
        }
    }
}

fn has_file_extension(file: &str, file_extensions: &[String]) -> bool {
    let file_low = file.to_lowercase();
    for ext in file_extensions {
        let ext_low = ext.to_lowercase();
        if file_low.ends_with(&ext_low) {
            return true;
        }
    }

    false
}