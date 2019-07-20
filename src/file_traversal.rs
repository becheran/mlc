extern crate walkdir;

use walkdir::WalkDir;

pub fn find<T: AsRef<str>>(root: &str, file_extensions: &[T], result: &mut Vec<String>) {
    info!("Search for files with extension '{}' in directory '{}'", to_string(file_extensions), root);
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

fn has_file_extension<T: AsRef<str>>(file: &str, file_extensions: &[T]) -> bool {
    let file_low = file.to_lowercase();
    for ext in file_extensions {
        let ext_low = ext.as_ref().to_lowercase();
        if file_low.ends_with(&ext_low) {
            return true;
        }
    }

    false
}

fn to_string<T: AsRef<str>>(file_extensions: &[T]) -> String {
    let mut result = String::new();
    result.push_str("[");
    for (i, ext) in file_extensions.iter().enumerate() {
        result.push_str("\"");
        result.push_str(ext.as_ref());
        if (i + 1) < file_extensions.len() {
            result.push_str("\", ");
        } else {
            result.push_str("\"");
        }
    }
    result.push_str("]");
    result
}