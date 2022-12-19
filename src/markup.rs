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
    pub fn file_extensions(&self) -> Vec<String> {
        match self {
            MarkupType::Markdown => vec![
                "md".to_string(),
                "markdown".to_string(),
                "mkdown".to_string(),
                "mkdn".to_string(),
                "mkd".to_string(),
                "mdwn".to_string(),
                "mdtxt".to_string(),
                "mdtext".to_string(),
                "text".to_string(),
                "rmd".to_string(),
            ],
            MarkupType::Html => vec!["htm".to_string(), "html".to_string(), "xhtml".to_string()],
        }
    }
}
