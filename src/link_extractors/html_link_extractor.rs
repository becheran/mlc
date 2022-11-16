use crate::link_extractors::link_extractor::LinkExtractor;
use crate::link_extractors::link_extractor::MarkupLink;

pub struct HtmlLinkExtractor();

#[derive(Clone, Copy, Debug)]
enum ParserState {
    Text,
    Comment,
    Anchor,
    EqualSign,
    Link,
}

impl LinkExtractor for HtmlLinkExtractor {
    fn find_links(&self, text: &str) -> Vec<MarkupLink> {
        let mut result: Vec<MarkupLink> = Vec::new();
        let mut state: ParserState = ParserState::Text;
        let mut link_column = 0;
        let mut link_line = 0;
        for (line, line_str) in text.lines().enumerate() {
            let line_chars: Vec<char> = line_str.chars().collect();
            let mut column: usize = 0;
            while line_chars.get(column).is_some() {
                match state {
                    ParserState::Comment => {
                        if line_chars.get(column) == Some(&'-')
                            && line_chars.get(column + 1) == Some(&'-')
                            && line_chars.get(column + 2) == Some(&'>')
                        {
                            column += 2;
                            state = ParserState::Text;
                        }
                    }
                    ParserState::Text => {
                        link_column = column;
                        link_line = line;
                        if line_chars.get(column) == Some(&'<')
                            && line_chars.get(column + 1) == Some(&'!')
                            && line_chars.get(column + 2) == Some(&'-')
                            && line_chars.get(column + 3) == Some(&'-')
                        {
                            column += 3;
                            state = ParserState::Comment;
                        } else if line_chars.get(column) == Some(&'<')
                            && line_chars.get(column + 1) == Some(&'a')
                        {
                            column += 1;
                            state = ParserState::Anchor;
                        }
                    }
                    ParserState::Anchor => {
                        if line_chars.get(column) == Some(&'h')
                            && line_chars.get(column + 1) == Some(&'r')
                            && line_chars.get(column + 2) == Some(&'e')
                            && line_chars.get(column + 3) == Some(&'f')
                        {
                            column += 3;
                            state = ParserState::EqualSign;
                        }
                    }
                    ParserState::EqualSign => {
                        match line_chars.get(column) {
                            Some(x) if x.is_whitespace() => {}
                            Some(x) if x == &'=' => state = ParserState::Link,
                            Some(_) => state = ParserState::Anchor,
                            None => {}
                        };
                    }
                    ParserState::Link => {
                        match line_chars.get(column) {
                            Some(x) if !x.is_whitespace() && x != &'"' => {
                                let start_col = column;
                                while line_chars.get(column).is_some()
                                    && !line_chars[column].is_whitespace()
                                    && line_chars[column] != '"'
                                {
                                    column += 1;
                                }
                                while let Some(c) = line_chars.get(column) {
                                    if c.is_whitespace() || c == &'"' {
                                        break;
                                    }
                                    column += 1;
                                }
                                let link =
                                    (line_chars[start_col..column]).iter().collect::<String>();
                                result.push(MarkupLink {
                                    column: link_column + 1,
                                    line: link_line + 1,
                                    target: link.to_string(),
                                    source: "".to_string(),
                                });
                                state = ParserState::Text;
                            }
                            Some(_) | None => {}
                        };
                    }
                }
                column += 1;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;

    #[test]
    fn no_link() {
        let le = HtmlLinkExtractor();
        let input = "]This is not a <has> no link <h1>Bla</h1> attribute.";
        let result = le.find_links(input);
        assert!(result.is_empty());
    }

    #[test]
    fn commented() {
        let le = HtmlLinkExtractor();
        let input = "df <!-- <a href=\"http://wiki.selfhtml.org\"> haha</a> -->";
        let result = le.find_links(input);
        assert!(result.is_empty());
    }

    #[test_case("<a href=\"https://www.w3schools.com\">Visit W3Schools.com!</a>", 1, 1)]
    #[test_case(
        "<a\nhref\n=\n  \"https://www.w3schools.com\">\nVisit W3Schools.com!\n</a>",
        1,
        1
    )]
    #[test_case(
        "<a hreflang=\"en\" href=\"https://www.w3schools.com\">Visit W3Schools.com!</a>",
        1,
        1
    )]
    #[test_case(
        "<!--comment--><a href=\"https://www.w3schools.com\">Visit W3Schools.com!</a>",
        1,
        15
    )]
    fn links(input: &str, line: usize, column: usize) {
        let le = HtmlLinkExtractor();
        let result = le.find_links(input);
        let expected = MarkupLink {
            target: "https://www.w3schools.com".to_string(),
            line,
            column,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }
}
