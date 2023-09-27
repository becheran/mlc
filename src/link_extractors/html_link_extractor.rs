use crate::link_extractors::link_extractor::LinkExtractor;
use crate::link_extractors::link_extractor::MarkupLink;
use crate::link_validator::link_type::get_link_type;
use crate::link_validator::link_type::LinkType;
pub struct HtmlLinkExtractor();

#[derive(Clone, Copy, Debug)]
enum ParserState {
    Text,
    Comment,
    Anchor,
    EqualSign,
    Link,
    Ignore,
}

impl HtmlLinkExtractor {
    pub fn find_links_with_ignore_info(&self, text: &str) -> (Vec<MarkupLink>, bool) {
        let mut result: Vec<MarkupLink> = Vec::new();
        let mut state: ParserState = ParserState::Text;
        let mut link_column = 0;
        let mut link_line = 0;
        for (line, line_str) in text.lines().enumerate() {
            let line_chars: Vec<char> = line_str.chars().collect();
            let mut column: usize = 0;
            if matches!(state, ParserState::Ignore) {
                info!("Ignore line {}", line);
                state = ParserState::Text;
                continue;
            }
            while line_chars.get(column).is_some() {
                match state {
                    ParserState::Ignore => {}
                    ParserState::Comment => {
                        if line_str.contains("mlc-ignore") {
                            info!("Ignore next link from line {}", line);
                            state = ParserState::Ignore;
                            break;
                        } else if line_chars.len() >= column + 3
                            && line_chars[column..column + 3] == ['-', '-', '>']
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
                                    if c == &'"' {
                                        break;
                                    }
                                    column += 1;
                                }
                                let mut link =
                                    (line_chars[start_col..column]).iter().collect::<String>();
                                if get_link_type(&link) == LinkType::FileSystem {
                                    link = url_escape::decode(link.as_str()).to_string();
                                };
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
        return (result, matches!(state, ParserState::Ignore));
    }
}

impl LinkExtractor for HtmlLinkExtractor {
    fn find_links(&self, text: &str) -> Vec<MarkupLink> {
        return self.find_links_with_ignore_info(text).0;
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

    #[test]
    fn space() {
        let le = HtmlLinkExtractor();
        let result = le.find_links("blah <a href=\"some file.html\">foo</a>.");
        let expected = MarkupLink {
            target: "some file.html".to_string(),
            line: 1,
            column: 6,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn url_encoded_path() {
        let le = HtmlLinkExtractor();
        let result = le.find_links("blah <a href=\"some%20file.html\">foo</a>.");
        let expected = MarkupLink {
            target: "some file.html".to_string(),
            line: 1,
            column: 6,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn ignore_next_line() {
        let le = HtmlLinkExtractor();
        let result =
            le.find_links("blah <!-- mlc-ignore -->\n<a href=\"some%20file.html\">foo</a>.");
        assert!(result.is_empty());
    }

    #[test_case("<!-- mlc-ignore -->\n<a href=\"https://foo.com\">Bar</a>\n<a href=\"https://www.w3schools.com\">Visit W3Schools.com!</a>", 3, 1)]
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
        "<!--comment--><a href=\"https://www.w3schools.com\">Visit W3Schools.com!</a><!--inf comment",
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
