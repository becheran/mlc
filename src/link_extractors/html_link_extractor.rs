use crate::link_extractors::link_extractor::LinkExtractor;
use crate::link_extractors::link_extractor::MarkupLink;

pub struct HtmlLinkExtractor();

fn skip_whitespace(vector: &Vec<char>, pos: &mut usize) {
    while *pos < vector.len() && vector[*pos].is_whitespace() {
        *pos += 1;
    }
}

fn forward_until(vector: &Vec<char>, pos: &mut usize, check_char: char) -> bool {
    while vector.get(*pos).is_some() && vector.get(*pos) != Some(&check_char) {
        *pos += 1;
    }
    vector.get(*pos).is_some()
}

impl LinkExtractor for HtmlLinkExtractor {
    fn find_links(&self, text: &str) -> Vec<MarkupLink> {
        let mut result: Vec<MarkupLink> = Vec::new();
        let mut is_comment_block = false;
        let mut is_anchor = false;
        for (line, line_str) in text.lines().enumerate() {
            let line_chars: Vec<char> = line_str.chars().collect();
            let mut column: usize = 0;

            if is_comment_block {
                if line_chars.get(column) == Some(&'-')
                    && line_chars.get(column + 1) == Some(&'-')
                    && line_chars.get(column + 2) == Some(&'>')
                {
                    column += 3;
                    is_comment_block = false;
                } else {
                    continue;
                }
            }  
            
            if is_anchor {
                if line_chars.get(column) == Some(&'h')
                    && line_chars.get(column + 1) == Some(&'r')
                    && line_chars.get(column + 2) == Some(&'e')
                    && line_chars.get(column + 3) == Some(&'f')
                {}

            } else {
                if forward_until(&line_chars, &mut column, '<') {
                    column += 1;
                }
                if line_chars.get(column) == Some(&'a') {
                    is_anchor = true;
                    column += 1;
                }
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
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn commented() {
        let le = HtmlLinkExtractor();
        let input = "df <!-- <a href=\"http://wiki.selfhtml.org\"> haha</a> -->";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test_case("<a href=\"https://www.w3schools.com\">Visit W3Schools.com!</a>", 4)]
    #[test_case(
        "<a\nhref\n=\n  \"https://www.w3schools.com\">\nVisit W3Schools.com!\n</a>",
        4
    )]
    #[test_case(
        "<a hreflang=\"en\" href=\"https://www.w3schools.com\">Visit W3Schools.com!</a>",
        4
    )]
    fn links(input: &str, column: usize) {
        let le = HtmlLinkExtractor();
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: "https://www.w3schools.com".to_string(),
            line: 1,
            column: column,
        };
        assert_eq!(vec![expected], result);
    }
}
