#[derive(Debug)]
pub struct MarkupFile{
    pub markup_type: MarkupType,
    pub path: String,
}

#[derive(Debug, Clone)]
pub enum MarkupType {
    Markdown,
    HTML,
}

impl MarkupType {
    pub fn file_extensions(&self) -> Vec<String> {
        match self {
            MarkupType::Markdown => vec!["md".to_string()],
            MarkupType::HTML => vec!["html".to_string(), "xhtml".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_type_file_endings() {
        let md = MarkupType::Markdown;
        assert_eq!(vec!["md".to_string()], md.file_extensions())
    }

    #[test]
    fn html_type_file_endings() {
        let md = MarkupType::HTML;
        assert_eq!(vec!["html".to_string(), "xhtml".to_string()], md.file_extensions())
    }
}