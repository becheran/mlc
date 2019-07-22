/// Links found in files
pub struct Link {
    /// The target the links points to
    pub target: String,
    /// The source file path
    pub source: String,
    /// The line number of the link destination starting from one
    pub line_nr: usize,
}