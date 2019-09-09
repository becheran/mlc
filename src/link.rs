use std::fmt;
use std::path::PathBuf;
use std::path::Path;

pub trait LinkTrait{
    fn absolute_target_path(&self) -> PathBuf;
}

/// Links found in files
pub struct Link {
    /// The target the links points to
    pub target: String,
    /// The source file path
    pub source: String,
    /// The line number of the link destination starting from one
    pub line_nr: usize,
}

impl fmt::Debug for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#{} => {}", self.source, self.line_nr, self.target)
    }
}

impl LinkTrait for Link {
    fn absolute_target_path(&self) -> PathBuf {
        if Path::new(&self.target).is_relative(){
            let parent = Path::new(&self.source).parent().unwrap_or(Path::new("./"));
            parent.join(&self.target)
        } else {
            Path::new(&self.target).to_path_buf()
        }
    }
}