/// Module for parsing ignore/disable comments in markup files
/// Supports comments like:
/// - `<!-- mlc-disable -->` / `<!-- mlc-enable -->`
/// - `<!-- mlc-disable-next-line -->`
/// - `<!-- mlc-disable-line -->`

#[derive(Debug, Clone, Copy, PartialEq)]
enum IgnoreState {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct IgnoreRegions {
    /// Lines that should be ignored (1-indexed)
    ignored_lines: Vec<usize>,
    /// Ranges of lines that should be ignored (1-indexed, inclusive)
    ignored_ranges: Vec<(usize, usize)>,
}

impl IgnoreRegions {
    /// Create a new IgnoreRegions from text content
    pub fn from_text(text: &str) -> Self {
        let mut ignored_lines = Vec::new();
        let mut ignored_ranges = Vec::new();
        let mut state = IgnoreState::Enabled;
        let mut disable_start_line = 0;

        for (line_idx, line) in text.lines().enumerate() {
            let line_num = line_idx + 1; // 1-indexed

            // Check for disable/enable blocks
            if line.contains("<!-- mlc-disable -->") {
                if state == IgnoreState::Enabled {
                    state = IgnoreState::Disabled;
                    disable_start_line = line_num;
                }
            } else if line.contains("<!-- mlc-enable -->") {
                if state == IgnoreState::Disabled {
                    // Add the range from disable to enable (exclusive of enable line)
                    ignored_ranges.push((disable_start_line, line_num));
                    state = IgnoreState::Enabled;
                }
            }

            // Check for single-line ignores
            if line.contains("<!-- mlc-disable-line -->") {
                ignored_lines.push(line_num);
            }

            // Check for next-line ignore
            if line.contains("<!-- mlc-disable-next-line -->") {
                ignored_lines.push(line_num + 1);
            }
        }

        // If we ended in disabled state, ignore from disable_start_line to end
        if state == IgnoreState::Disabled {
            let total_lines = text.lines().count();
            if total_lines > 0 {
                ignored_ranges.push((disable_start_line, total_lines + 1));
            }
        }

        Self {
            ignored_lines,
            ignored_ranges,
        }
    }

    /// Check if a given line number (1-indexed) should be ignored
    pub fn is_line_ignored(&self, line: usize) -> bool {
        // Check if line is in ignored_lines
        if self.ignored_lines.contains(&line) {
            return true;
        }

        // Check if line is in any ignored range
        for (start, end) in &self.ignored_ranges {
            if line >= *start && line <= *end {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_ignore_comments() {
        let text = "This is a normal line\nAnother line";
        let regions = IgnoreRegions::from_text(text);
        assert!(!regions.is_line_ignored(1));
        assert!(!regions.is_line_ignored(2));
    }

    #[test]
    fn disable_line_comment() {
        let text = "Line 1\n<!-- mlc-disable-line --> Line 2\nLine 3";
        let regions = IgnoreRegions::from_text(text);
        assert!(!regions.is_line_ignored(1));
        assert!(regions.is_line_ignored(2));
        assert!(!regions.is_line_ignored(3));
    }

    #[test]
    fn disable_next_line_comment() {
        let text = "Line 1\n<!-- mlc-disable-next-line -->\nLine 3\nLine 4";
        let regions = IgnoreRegions::from_text(text);
        assert!(!regions.is_line_ignored(1));
        assert!(!regions.is_line_ignored(2));
        assert!(regions.is_line_ignored(3));
        assert!(!regions.is_line_ignored(4));
    }

    #[test]
    fn disable_enable_block() {
        let text = "Line 1\n<!-- mlc-disable -->\nLine 3\nLine 4\n<!-- mlc-enable -->\nLine 6";
        let regions = IgnoreRegions::from_text(text);
        assert!(!regions.is_line_ignored(1));
        assert!(regions.is_line_ignored(2));
        assert!(regions.is_line_ignored(3));
        assert!(regions.is_line_ignored(4));
        assert!(regions.is_line_ignored(5));
        assert!(!regions.is_line_ignored(6));
    }

    #[test]
    fn disable_without_enable() {
        let text = "Line 1\nLine 2\n<!-- mlc-disable -->\nLine 4\nLine 5";
        let regions = IgnoreRegions::from_text(text);
        assert!(!regions.is_line_ignored(1));
        assert!(!regions.is_line_ignored(2));
        assert!(regions.is_line_ignored(3));
        assert!(regions.is_line_ignored(4));
        assert!(regions.is_line_ignored(5));
    }

    #[test]
    fn multiple_disable_blocks() {
        let text = "Line 1\n<!-- mlc-disable -->\nLine 3\n<!-- mlc-enable -->\nLine 5\n<!-- mlc-disable -->\nLine 7\n<!-- mlc-enable -->\nLine 9";
        let regions = IgnoreRegions::from_text(text);
        assert!(!regions.is_line_ignored(1));
        assert!(regions.is_line_ignored(2));
        assert!(regions.is_line_ignored(3));
        assert!(regions.is_line_ignored(4));
        assert!(!regions.is_line_ignored(5));
        assert!(regions.is_line_ignored(6));
        assert!(regions.is_line_ignored(7));
        assert!(regions.is_line_ignored(8));
        assert!(!regions.is_line_ignored(9));
    }

    #[test]
    fn mixed_ignore_types() {
        let text = "Line 1\n<!-- mlc-disable-line --> Line 2\n<!-- mlc-disable-next-line -->\nLine 4\n<!-- mlc-disable -->\nLine 6\n<!-- mlc-enable -->\nLine 8";
        let regions = IgnoreRegions::from_text(text);
        assert!(!regions.is_line_ignored(1));
        assert!(regions.is_line_ignored(2)); // disable-line
        assert!(!regions.is_line_ignored(3));
        assert!(regions.is_line_ignored(4)); // disable-next-line
        assert!(regions.is_line_ignored(5)); // disable block start
        assert!(regions.is_line_ignored(6)); // disable block
        assert!(regions.is_line_ignored(7)); // disable block end (enable)
        assert!(!regions.is_line_ignored(8));
    }
}
