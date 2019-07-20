extern crate walkdir;

use walkdir::WalkDir;

pub fn find(root: &str, file_extensions: &[&str], result: &mut Vec<String>) {
    info!("Search for files with extension {:?} in directory '{}'", file_extensions, root);
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();
        if has_file_extension(&f_name, &file_extensions) {
            debug!("Found file: {}", f_name);
            //result.push(312);
            result.push(f_name.to_string());
        }
    }
}

fn has_file_extension(file: &str, file_extensions: &[&str]) -> bool {
    let file_low = file.to_lowercase();
    for ext in file_extensions {
        let ext_low = ext.to_lowercase();
        if file_low.ends_with(&ext_low) {
            return true;
        }
    }

    false
}