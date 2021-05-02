#[cfg(test)]

use std::path::{Path, PathBuf};

pub fn benches_dir()-> PathBuf {
    Path::new(file!())
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .join("benches")
}