use crate::link_extractors::link_extractor::LinkExtractor;
use crate::link_extractors::link_extractor::MarkupLink;

pub struct MarkdownLinkExtractor();

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

impl LinkExtractor for MarkdownLinkExtractor {
    fn find_links(&self, text: &str) -> Vec<MarkupLink> {
        let mut result: Vec<MarkupLink> = Vec::new();
        let mut link_tags: Vec<String> = Vec::new();
        let mut reference_link_tags: Vec<String> = Vec::new();
        let mut is_code_block = false;
        for (line, line_str) in text.lines().enumerate() {
            let line_chars: Vec<char> = line_str.chars().collect();
            let mut column: usize = 0;

            skip_whitespace(&line_chars, &mut column);

            if line_chars.get(column) == Some(&'#') {
                continue;
            }

            if line_chars.get(column) == Some(&'`')
                && line_chars.get(column + 1) == Some(&'`')
                && line_chars.get(column + 2) == Some(&'`')
            {
                is_code_block = !is_code_block;
                column += 3;
            }

            if is_code_block {
                continue;
            }

            while column < line_chars.len() {
                match line_chars[column] {
                    '`' => {
                        column += 1;
                        forward_until(&line_chars, &mut column, '`');
                    }
                    '\\' => {
                        column += 1; // Escape next character
                    }
                    '[' => {
                        let square_bracket_start = column;
                        if forward_until(&line_chars, &mut column, ']') {
                            let square_bracket_close = column;
                            column += 1;
                            let start_idx = column + 1;
                            match line_chars.get(column) {
                                None => {
                                    let reference_link_tag = (&line_chars
                                        [square_bracket_start + 1..square_bracket_close])
                                        .iter()
                                        .collect::<String>();
                                    debug!(
                                        "Found reference link of format [] {:?}",
                                        reference_link_tags
                                    );
                                    reference_link_tags.push(reference_link_tag.to_lowercase());
                                }
                                Some(&'(') => {
                                    let bracket_start = column;
                                    column += 1;
                                    if forward_until(&line_chars, &mut column, ')') {
                                        let link = (&line_chars[start_idx..column])
                                            .iter()
                                            .collect::<String>();
                                        // Take first split because of possible title tag
                                        let mut spl = link.split_whitespace();
                                        let link = spl.next().unwrap_or("");
                                        debug!("Extract link link in format []() {:?}", link);
                                        result.push(MarkupLink {
                                            column: bracket_start + 2,
                                            line: line + 1,
                                            target: link.to_string(),
                                            source: "".to_string(),
                                        });
                                    }
                                }
                                Some(&'[') => {
                                    column += 1;
                                    if forward_until(&line_chars, &mut column, ']') {
                                        let reference_link = (&line_chars[start_idx..column])
                                            .iter()
                                            .collect::<String>();
                                        debug!("Extract reference link {:?}", reference_link);
                                        link_tags.push(reference_link.to_lowercase());
                                    }
                                }
                                Some(&':') => {
                                    column += 1;
                                    skip_whitespace(&line_chars, &mut column);
                                    let start_idx = column;
                                    while column < line_chars.len() && !line_chars[column].is_whitespace() {
                                        column += 1;
                                    }
                                    let link =
                                        (&line_chars[start_idx..column]).iter().collect::<String>();
                                    result.push(MarkupLink {
                                        column: start_idx + 1,
                                        line: line + 1,
                                        target: link.to_string(),
                                        source: "".to_string(),
                                    });
                                    debug!("Extract link of format []: {:?}", link);
                                    let reference_link_tag = (&line_chars
                                        [square_bracket_start + 1..square_bracket_close])
                                        .iter()
                                        .collect::<String>();
                                    debug!("Found reference link tag {:?}", reference_link_tags);
                                    reference_link_tags.push(reference_link_tag.to_lowercase());
                                }
                                Some(_) => {}
                            }
                        }
                    }
                    ':' => {
                        if line_chars.get(column + 1) == Some(&'/')
                            && line_chars.get(column + 2) == Some(&'/')
                        {
                            let mut start_idx = column;
                            while start_idx > 0 && line_chars[start_idx - 1].is_alphabetic() {
                                start_idx -= 1;
                            }
                            while column < line_chars.len()
                                && !(line_chars[column].is_whitespace()
                                    || line_chars[column] == '>'
                                    || line_chars[column] == ')'
                                    || line_chars[column] == ']')
                            {
                                column += 1;
                            }
                            let link = (&line_chars[start_idx..column]).iter().collect::<String>();
                            result.push(MarkupLink {
                                column: start_idx + 1,
                                line: line + 1,
                                target: link.to_string(),
                                source: "".to_string(),
                            });
                        }
                    }
                    '<' => {
                        let link_column = column;
                        column += 1;
                        if line_chars.get(column) == Some(&'a') {
                            column += 1;
                            skip_whitespace(&line_chars, &mut column);
                            if line_chars.get(column) == Some(&'h')
                                && line_chars.get(column + 1) == Some(&'r')
                                && line_chars.get(column + 2) == Some(&'e')
                                && line_chars.get(column + 3) == Some(&'f')
                            {
                                column += 4;
                                skip_whitespace(&line_chars, &mut column);
                                if line_chars.get(column) == Some(&'=') {
                                    column += 1;
                                    while column < line_chars.len()
                                        && (line_chars[column].is_whitespace()
                                            || line_chars[column] == '"')
                                    {
                                        column += 1;
                                    }
                                    let start_idx = column;
                                    while line_chars.get(column).is_some()
                                        && !line_chars[column].is_whitespace()
                                        && line_chars[column] != '"'
                                    {
                                        column += 1;
                                    }
                                    let link =
                                        (&line_chars[start_idx..column]).iter().collect::<String>();
                                    result.push(MarkupLink {
                                        column: link_column + 1,
                                        line: line + 1,
                                        target: link.to_string(),
                                        source: "".to_string(),
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
                column += 1;
            }
        }
        for link in link_tags {
            if !reference_link_tags.contains(&link) {
                warn!("No link for reference [{}] found.", link);
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
    fn inline_no_link() {
        let le = MarkdownLinkExtractor();
        let input = "]This is not a () link](! has no title attribute.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn nested_links() {
        let le = MarkdownLinkExtractor();
        let input = "[![](http://meritbadge.herokuapp.com/mlc)](https://crates.io/crates/mlc)";
        let result = le.find_links(&input);
        let img = MarkupLink {
            target: "http://meritbadge.herokuapp.com/mlc".to_string(),
            line: 1,
            column: 6,
            source: "".to_string(),
        };
        let link = MarkupLink {
            target: "https://crates.io/crates/mlc".to_string(),
            line: 1,
            column: 44,
            source: "".to_string(),
        };
        assert_eq!(vec![img, link], result);
    }

    #[test]
    fn link_escaped() {
        let le = MarkdownLinkExtractor();
        let input = format!("This is not a \\[link\\](random_link).");
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn link_in_headline() {
        let le = MarkdownLinkExtractor();
        let input = format!("  # This is not a [link](http://example.net/).");
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn code_block() {
        let le = MarkdownLinkExtractor();
        let input = format!(" `[code](http://example.net/)`, no link!.");
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn escaped_code_block() {
        let le = MarkdownLinkExtractor();
        let input = format!("   klsdjf \\`[escape](http://example.net/)\\`, no link!.");
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column: 22,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_in_code_block() {
        let le = MarkdownLinkExtractor();
        let input = format!("```\n[only code](http://example.net/)\n```.");
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn image_reference() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("\n\nBla ![This is an image link]({})", link_str);
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: link_str.to_string(),
            line: 3,
            column: 30,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_no_title() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("[This link]({}) has no title attribute.", link_str);
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: link_str.to_string(),
            line: 1,
            column: 13,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_with_title() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("\n123[This is a link]({} \"with title\") oh yea.", link_str);
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: link_str.to_string(),
            line: 2,
            column: 21,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test_case("<http://example.net/>", 2)]
    #[test_case("http://example.net/", 1)]
    #[test_case("This is a short link http://example.net/", 22)]
    #[test_case("This is a short link <http://example.net/>", 23)]
    fn inline_link(input: &str, column: usize) {
        let le = MarkdownLinkExtractor();
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column: column,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test_case(
        "<a href=\"http://example.net/\"> target=\"_blank\">Visit W3Schools!</a>",
        test_name = "html_link_with_target"
    )]
    #[test_case(
        "<a href=\"http://example.net/\"> link text</a>",
        test_name = "html_link_no_target"
    )]
    fn html_link(input: &str) {
        let le = MarkdownLinkExtractor();
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column: 1,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn referenced_link() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("[I'm a reference-style link][Arbitrary case-insensitive reference text].\n[arbitrary case-insensitive reference text]: {}", link_str);
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: link_str.to_string(),
            line: 2,
            column: 46,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn referenced_link_tag_only() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!(
            "[arbitrary case-insensitive reference text].\n[arbitrary case-insensitive reference text]: {}",
            link_str
        );
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: link_str.to_string(),
            line: 2,
            column: 46,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }
}
