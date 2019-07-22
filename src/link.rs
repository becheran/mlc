use std::fmt;

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