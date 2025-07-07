use super::html_link_extractor::HtmlLinkExtractor;
use super::link_extractor::BrokenExtractedLink;
use crate::link_extractors::link_extractor::LinkExtractor;
use crate::link_extractors::link_extractor::MarkupLink;
use pulldown_cmark::{BrokenLink, Event, Options, Parser, Tag};

pub struct MarkdownLinkExtractor();

impl LinkExtractor for MarkdownLinkExtractor {
    fn find_links(&self, text: &str) -> Vec<Result<MarkupLink, BrokenExtractedLink>> {
        use std::cell::RefCell;
        let result: RefCell<Vec<Result<MarkupLink, BrokenExtractedLink>>> =
            RefCell::new(Vec::new());

        let html_extractor = HtmlLinkExtractor();
        let converter = LineColumnConverter::new(text);

        let callback = &mut |broken_link: BrokenLink| {
            let line_col = converter.line_column_from_idx(broken_link.span.start);
            info!(
                "Broken link in md file: {} (line {}, column {})",
                broken_link.reference, line_col.0, line_col.1
            );
            result.borrow_mut().push(Err(BrokenExtractedLink {
                source: String::new(),
                line: line_col.0,
                column: line_col.1,
                reference: broken_link.reference.to_string(),
                error: "Markdown reference not found".to_string(),
            }));
            None
        };

        let parser = Parser::new_with_broken_link_callback(text, Options::empty(), Some(callback));

        for (evt, range) in parser.into_offset_iter() {
            match evt {
                Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => {
                    let line_col = converter.line_column_from_idx(range.start);
                    result.borrow_mut().push(Ok(MarkupLink {
                        line: line_col.0,
                        column: line_col.1,
                        source: String::new(),
                        target: dest_url.to_string(),
                    }));
                }
                Event::Html(html) | Event::InlineHtml(html) => {
                    let line_col = converter.line_column_from_idx(range.start);
                    let html_result = html_extractor.find_links(html.as_ref());
                    let mut parsed_html = html_result
                        .iter()
                        .filter_map(|res| res.as_ref().ok())
                        .map(|md_link| {
                            let line = line_col.0 + md_link.line - 1;
                            let column = if md_link.line > 1 {
                                md_link.column
                            } else {
                                line_col.1 + md_link.column - 1
                            };
                            Ok(MarkupLink {
                                column,
                                line,
                                source: md_link.source.clone(),
                                target: md_link.target.clone(),
                            })
                        })
                        .collect();
                    result.borrow_mut().append(&mut parsed_html);
                }
                _ => (),
            };
        }
        result.into_inner()
    }
}

struct LineColumnConverter {
    line_lengths: Vec<usize>,
}

impl LineColumnConverter {
    fn new(text: &str) -> Self {
        let mut line_lengths: Vec<usize> = Vec::new();
        let mut current_line_len = 0;
        for c in text.chars() {
            current_line_len += c.len_utf8();
            if c == '\n' {
                line_lengths.push(current_line_len);
                current_line_len = 0;
            }
        }
        Self { line_lengths }
    }

    fn line_column_from_idx(&self, idx: usize) -> (usize, usize) {
        let mut line = 1;
        let mut column = idx + 1;
        for line_length in &self.line_lengths {
            if *line_length >= column {
                return (line, column);
            }
            column -= line_length;
            line += 1;
        }
        (line, column)
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
        let result = le.find_links(input);
        assert!(result.is_empty());
    }

    #[test]
    fn commented_link() {
        let le = MarkdownLinkExtractor();
        let input = "]This is not a () <!--[link](link)-->.";
        let result = le.find_links(input);
        assert!(result.is_empty());
    }

    #[test]
    fn nested_links() {
        let le = MarkdownLinkExtractor();
        let input =
            "\n\r\t\n[![](http://meritbadge.herokuapp.com/mlc)](https://crates.io/crates/mlc)";
        let result = le.find_links(input);
        let img = Ok(MarkupLink {
            target: "http://meritbadge.herokuapp.com/mlc".to_string(),
            line: 3,
            column: 2,
            source: "".to_string(),
        });
        let link = Ok(MarkupLink {
            target: "https://crates.io/crates/mlc".to_string(),
            line: 3,
            column: 1,
            source: "".to_string(),
        });
        assert_eq!(vec![link, img], result);
    }

    #[test]
    fn link_escaped() {
        let le = MarkdownLinkExtractor();
        let input = "This is not a \\[link\\](random_link).";
        let result = le.find_links(input);
        assert!(result.is_empty());
    }

    #[test]
    fn link_in_headline() {
        let le = MarkdownLinkExtractor();
        let input = "  # This is a [link](http://example.net/).";
        let result = le.find_links(input);
        assert_eq!(result[0].as_ref().unwrap().column, 15);
    }

    #[test]
    fn no_link_colon() {
        let le = MarkdownLinkExtractor();
        let input = "This is not a [link:bla.";
        let result = le.find_links(input);
        assert!(result.is_empty());
    }

    #[test]
    fn broken_reference_link() {
        let le = MarkdownLinkExtractor();
        let input = "This is not a [link]:bla.";
        let result = le.find_links(input);

        let expected = Err(BrokenExtractedLink {
            source: "".to_string(),
            reference: "link".to_string(),
            line: 1,
            column: 15,
            error: "Markdown reference not found".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn inline_code() {
        let le = MarkdownLinkExtractor();
        let input = " `[code](http://example.net/)`, no link!.";
        let result = le.find_links(input);
        assert!(result.is_empty());
    }

    #[test]
    fn link_near_inline_code() {
        let le = MarkdownLinkExtractor();
        let input = " `bug` [code](http://example.net/), link!.";
        let result = le.find_links(input);
        let expected = Ok(MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column: 8,
            source: "".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_very_near_inline_code() {
        let le = MarkdownLinkExtractor();
        let input = "`bug`[code](http://example.net/)";
        let result = le.find_links(input);
        let expected = Ok(MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column: 6,
            source: "".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn code_block() {
        let le = MarkdownLinkExtractor();
        let input = " ``` js\n[code](http://example.net/)```, no link!.";
        let result = le.find_links(input);
        assert!(result.is_empty());
    }

    #[test]
    fn html_code_block() {
        let le = MarkdownLinkExtractor();
        let input = "<script>\n[code](http://example.net/)</script>, no link!.";
        let result = le.find_links(input);
        assert!(result.is_empty());
    }

    #[test]
    fn escaped_code_block() {
        let le = MarkdownLinkExtractor();
        let input = "   klsdjf \\`[escape](http://example.net/)\\`, no link!.";
        let result = le.find_links(input);
        let expected = Ok(MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column: 13,
            source: "".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_in_code_block() {
        let le = MarkdownLinkExtractor();
        let input = "```\n[only code](http://example.net/)\n```.";
        let result = le.find_links(input);
        assert!(result.is_empty());
    }

    #[test]
    fn image_reference() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("\n\nBla ![This is an image link]({link_str})");
        let result = le.find_links(&input);
        let expected = Ok(MarkupLink {
            target: link_str.to_string(),
            line: 3,
            column: 5,
            source: "".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_no_title() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("[This link]({link_str}) has no title attribute.");
        let result = le.find_links(&input);
        let expected = Ok(MarkupLink {
            target: link_str.to_string(),
            line: 1,
            column: 1,
            source: "".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_with_title() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("\n123[This is a link]({link_str} \"with title\") oh yea.");
        let result = le.find_links(&input);
        let expected = Ok(MarkupLink {
            target: link_str.to_string(),
            line: 2,
            column: 4,
            source: "".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test_case("<http://example.net/>", 1)]
    // TODO GitHub Link style support
    //#[test_case("This is a short link http://example.net/", 22)]
    //#[test_case("http://example.net/", 1)]
    #[test_case("This is a short link <http://example.net/>", 22)]
    fn inline_link(input: &str, column: usize) {
        let le = MarkdownLinkExtractor();
        let result = le.find_links(input);
        let expected = Ok(MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column,
            source: "".to_string(),
        });
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
        let result = le.find_links(input);
        let expected = Ok(MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column: 1,
            source: "".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn html_link_ident() {
        let le = MarkdownLinkExtractor();
        let result = le.find_links("123<a href=\"http://example.net/\"> link text</a>");
        let expected = Ok(MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column: 4,
            source: "".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn html_link_new_line() {
        let le = MarkdownLinkExtractor();
        let result = le.find_links("\n123<a href=\"http://example.net/\"> link text</a>");
        let expected = Ok(MarkupLink {
            target: "http://example.net/".to_string(),
            line: 2,
            column: 4,
            source: "".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn raw_html_issue_31() {
        let le = MarkdownLinkExtractor();
        let result = le.find_links("Some text <a href=\"some_url\">link text</a> more text.");
        let expected = Ok(MarkupLink {
            target: "some_url".to_string(),
            line: 1,
            column: 11,
            source: "".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn referenced_link() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!(
            "This is [an example][arbitrary case-insensitive reference text] reference-style link.\n\n[Arbitrary CASE-insensitive reference text]: {link_str}"
        );
        let result = le.find_links(&input);
        let expected = Ok(MarkupLink {
            target: link_str.to_string(),
            line: 1,
            column: 9,
            source: "".to_string(),
        });
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn referenced_link_tag_only() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!(
            "Foo Bar\n\n[Arbitrary CASE-insensitive reference text]: {link_str}"
        );
        let result = le.find_links(&input);
        assert_eq!(0, result.len());
    }

    #[test]
    fn referenced_link_no_tag_only() {
        let le = MarkdownLinkExtractor();
        let input = "[link][reference]";
        let result = le.find_links(input);
        assert_eq!(1, result.len());
    }
}
