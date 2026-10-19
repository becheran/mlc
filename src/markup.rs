use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug)]
pub struct MarkupFile {
    pub markup_type: MarkupType,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum MarkupType {
    Markdown,
    Html,
}

impl FromStr for MarkupType {
    type Err = ();

    fn from_str(s: &str) -> Result<MarkupType, ()> {
        match s {
            "md" => Ok(MarkupType::Markdown),
            "html" => Ok(MarkupType::Html),
            _ => Err(()),
        }
    }
}

impl MarkupType {
    #[must_use]
    pub fn file_extensions(&self) -> &'static [&'static str] {
        match self {
            MarkupType::Markdown => &[
                "md", "markdown", "mkdown", "mkdn", "mkd", "mdwn", "mdtxt", "mdtext", "text", "rmd",
            ],
            MarkupType::Html => &["htm", "html", "xhtml"],
        }
    }

    /// The markup type of the first of `markup_types` which has the extension of `file_name`.
    #[must_use]
    pub fn from_file_name(file_name: &str, markup_types: &[MarkupType]) -> Option<MarkupType> {
        let (_, extension) = file_name.rsplit_once('.')?;
        markup_types.iter().copied().find(|markup_type| {
            markup_type
                .file_extensions()
                .iter()
                .any(|ext| ext.eq_ignore_ascii_case(extension))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [MarkupType; 2] = [MarkupType::Markdown, MarkupType::Html];

    #[test]
    fn markup_type_from_file_name() {
        let markup_type = |name| MarkupType::from_file_name(name, &ALL);
        assert!(matches!(
            markup_type("README.md"),
            Some(MarkupType::Markdown)
        ));
        assert!(matches!(
            markup_type("README.MD"),
            Some(MarkupType::Markdown)
        ));
        assert!(matches!(
            markup_type("a.b.markdown"),
            Some(MarkupType::Markdown)
        ));
        assert!(matches!(markup_type(".md"), Some(MarkupType::Markdown)));
        assert!(matches!(markup_type("index.xhtml"), Some(MarkupType::Html)));
        assert!(markup_type("md").is_none());
        assert!(markup_type("context").is_none());
        assert!(markup_type("notes.txt").is_none());
        assert!(markup_type("dir.md/file").is_none());
    }

    #[test]
    fn markup_type_from_file_name_respects_selected_types() {
        assert!(MarkupType::from_file_name("a.html", &[MarkupType::Markdown]).is_none());
        assert!(MarkupType::from_file_name("a.html", &[]).is_none());
    }
}
