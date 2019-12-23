use crate::link_extractors::link_extractor::LinkExtractor;
use crate::link_extractors::link_extractor::MarkupLink;

pub struct MarkdownLinkExtractor();

impl LinkExtractor for MarkdownLinkExtractor {
    fn find_links(&self, text: &str) -> Vec<MarkupLink> {
        let mut result: Vec<MarkupLink> = Vec::new();
        let mut is_code_block = false;
        for (line, line_str) in text.lines().enumerate() {
            let line_chars: Vec<char> = line_str.chars().collect();
            let len = line_chars.len();
            let mut column = 0;
            for c in &line_chars {
                if !c.is_whitespace() {
                    break;
                }
                column += 1;
            }

            if column < len && line_chars[column] == '#' {
                continue;
            }

            if column + 2 < len
                && line_chars[column] == '`'
                && line_chars[column + 1] == '`'
                && line_chars[column + 2] == '`'
            {
                is_code_block = !is_code_block;
                column += 3;
            }

            if is_code_block {
                continue;
            }

            while column < len {
                if line_chars[column] == '`' {
                    column += 1;
                    while column < len && line_chars[column] != '`' {
                        column += 1;
                    }
                } else if line_chars[column] == '\\' {
                    column += 1; // Escape next character
                } else if line_chars[column] == '[' {
                    let link_column = column;
                    while column < len && line_chars[column] != ']' {
                        column += 1;
                    }
                    if column + 1 < len
                        && line_chars[column] == ']'
                        && line_chars[column + 1] == '('
                    {
                        column += 1;
                        let start_idx = column + 1;
                        while column < len && line_chars[column] != ')' {
                            column += 1;
                        }
                        if column < len && line_chars[column] == ')' {
                            let link = (&line_chars[start_idx..column])
                                .iter()
                                .cloned()
                                .collect::<String>();
                            // Take first split because of possible title tag
                            let mut spl = link.split_whitespace();
                            let link = spl.next().unwrap_or("");
                            result.push(MarkupLink {
                                column: link_column + 1,
                                line: line + 1,
                                target: link.to_string(),
                            });
                        }
                    }
                } else if line_chars[column] == ':' {
                    if column + 2 < len
                        && line_chars[column + 1] == '/'
                        && line_chars[column + 2] == '/'
                    {
                        let mut start_idx = column;
                        while start_idx > 0 && !line_chars[start_idx].is_whitespace() && line_chars[start_idx] != '<'  {
                            start_idx -= 1;
                        }
                        start_idx += 1;
                        while column < len && !line_chars[column].is_whitespace() && line_chars[column] != '>' {
                            column += 1;
                        }
                        let link = (&line_chars[start_idx..column])
                            .iter()
                            .cloned()
                            .collect::<String>();                        
                        result.push(MarkupLink {
                            column: start_idx + 1,
                            line: line + 1,
                            target: link.to_string(),
                        });
                    }
                //TODO Reference links
                } else if line_chars[column] == '<' {
                    //TODO html <a href=\"http://example.net/\">
                    let link_column = column;
                    column += 1;
                    if column < len && line_chars[column] == 'a' {
                        column += 1;
                        while column < len && line_chars[column].is_whitespace() {
                            column += 1;
                        }
                        if column + 3 < len
                            && line_chars[column] == 'h'
                            && line_chars[column + 1] == 'r'
                            && line_chars[column + 2] == 'e'
                            && line_chars[column + 3] == 'f'
                        {
                            column += 4;
                            while column < len && line_chars[column].is_whitespace() {
                                column += 1;
                            }
                            if column < len && line_chars[column] == '=' {
                                column += 1;
                                while column < len
                                    && (line_chars[column].is_whitespace()
                                        || line_chars[column] == '"')
                                {
                                    column += 1;
                                }
                                let start_idx = column;
                                while column < len
                                    && !line_chars[column].is_whitespace()
                                    && line_chars[column] != '"'
                                {
                                    column += 1;
                                }
                                let link = (&line_chars[start_idx..column])
                                    .iter()
                                    .cloned()
                                    .collect::<String>();
                                result.push(MarkupLink {
                                    column: link_column + 1,
                                    line: line + 1,
                                    target: link.to_string(),
                                });
                            }
                        }
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
    fn inline_no_link() {
        let le = MarkdownLinkExtractor();
        let input = "]This is not a () link](! has no title attribute.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
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
            column: 13,
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
            column: 6,
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
            column: 1,
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
            column: 4,
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn inline_link() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("This is a short link {} .", link_str);
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: link_str.to_string(),
            line: 1,
            column: 22,
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn inline_link_with_brackets() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("This is a short link <{}>.", link_str);
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: link_str.to_string(),
            line: 1,
            column: 23,
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
        };
        assert_eq!(vec![expected], result);
    }
}
