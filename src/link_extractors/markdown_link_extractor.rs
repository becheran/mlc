use super::html_link_extractor::HtmlLinkExtractor;
use super::link_extractor::BrokenExtractedLink;
use crate::link_extractors::link_extractor::LinkExtractor;
use crate::link_extractors::link_extractor::MarkupLink;
use crate::Config;
use pulldown_cmark::{BrokenLink, Event, Options, Parser, Tag};
use regex::Regex;

lazy_static! {
    // Regex to match HTTP(S) URLs in code blocks
    // Matches URLs including those with parentheses, brackets, and query parameters
    // The pattern matches greedily but we trim trailing punctuation in code
    static ref CODE_BLOCK_URL_REGEX: Regex = 
        Regex::new(r"https?://[^\s<>]+").unwrap();
}

pub struct MarkdownLinkExtractor();

impl LinkExtractor for MarkdownLinkExtractor {
    fn find_links(&self, text: &str, config: &Config) -> Vec<Result<MarkupLink, BrokenExtractedLink>> {
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

        let check_code_blocks = config.optional.check_links_in_code_blocks.unwrap_or(false);
        
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
                    let html_result = html_extractor.find_links(html.as_ref(), config);
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
                Event::Text(code_text) | Event::Code(code_text) if check_code_blocks => {
                    // Extract HTTP(S) URLs from code blocks using the pre-compiled regex
                    for url_match in CODE_BLOCK_URL_REGEX.find_iter(code_text.as_ref()) {
                        let mut url = url_match.as_str();
                        
                        // Trim common trailing punctuation that's likely not part of the URL
                        // But keep parentheses and brackets if they appear to be balanced
                        while let Some(last_char) = url.chars().last() {
                            if matches!(last_char, '.' | ',' | ';' | '!' | '?') {
                                url = &url[..url.len() - last_char.len_utf8()];
                            } else {
                                break;
                            }
                        }
                        
                        if url.is_empty() {
                            continue;
                        }
                        
                        let url_start_idx = range.start + url_match.start();
                        let url_line_col = converter.line_column_from_idx(url_start_idx);
                        result.borrow_mut().push(Ok(MarkupLink {
                            line: url_line_col.0,
                            column: url_line_col.1,
                            source: String::new(),
                            target: url.to_string(),
                        }));
                    }
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
    use crate::OptionalConfig;
    use ntest::test_case;
    use std::path::PathBuf;

    fn default_config() -> Config {
        Config {
            directory: PathBuf::from("."),
            optional: OptionalConfig::default(),
        }
    }

    fn config_with_code_blocks() -> Config {
        Config {
            directory: PathBuf::from("."),
            optional: OptionalConfig {
                check_links_in_code_blocks: Some(true),
                ..OptionalConfig::default()
            },
        }
    }

    #[test]
    fn inline_no_link() {
        let le = MarkdownLinkExtractor();
        let input = "]This is not a () link](! has no title attribute.";
        let result = le.find_links(input, &default_config());
        assert!(result.is_empty());
    }

    #[test]
    fn commented_link() {
        let le = MarkdownLinkExtractor();
        let input = "]This is not a () <!--[link](link)-->.";
        let result = le.find_links(input, &default_config());
        assert!(result.is_empty());
    }

    #[test]
    fn nested_links() {
        let le = MarkdownLinkExtractor();
        let input =
            "\n\r\t\n[![](http://meritbadge.herokuapp.com/mlc)](https://crates.io/crates/mlc)";
        let result = le.find_links(input, &default_config());
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
        let result = le.find_links(input, &default_config());
        assert!(result.is_empty());
    }

    #[test]
    fn link_in_headline() {
        let le = MarkdownLinkExtractor();
        let input = "  # This is a [link](http://example.net/).";
        let result = le.find_links(input, &default_config());
        assert_eq!(result[0].as_ref().unwrap().column, 15);
    }

    #[test]
    fn no_link_colon() {
        let le = MarkdownLinkExtractor();
        let input = "This is not a [link:bla.";
        let result = le.find_links(input, &default_config());
        assert!(result.is_empty());
    }

    #[test]
    fn broken_reference_link() {
        let le = MarkdownLinkExtractor();
        let input = "This is not a [link]:bla.";
        let result = le.find_links(input, &default_config());

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
        let result = le.find_links(input, &default_config());
        assert!(result.is_empty());
    }

    #[test]
    fn link_near_inline_code() {
        let le = MarkdownLinkExtractor();
        let input = " `bug` [code](http://example.net/), link!.";
        let result = le.find_links(input, &default_config());
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
        let result = le.find_links(input, &default_config());
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
        let result = le.find_links(input, &default_config());
        assert!(result.is_empty());
    }

    #[test]
    fn html_code_block() {
        let le = MarkdownLinkExtractor();
        let input = "<script>\n[code](http://example.net/)</script>, no link!.";
        let result = le.find_links(input, &default_config());
        assert!(result.is_empty());
    }

    #[test]
    fn escaped_code_block() {
        let le = MarkdownLinkExtractor();
        let input = "   klsdjf \\`[escape](http://example.net/)\\`, no link!.";
        let result = le.find_links(input, &default_config());
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
        let result = le.find_links(input, &default_config());
        assert!(result.is_empty());
    }

    #[test]
    fn image_reference() {
        let le = MarkdownLinkExtractor();
        let link_str = "http://example.net/";
        let input = format!("\n\nBla ![This is an image link]({link_str})");
        let result = le.find_links(&input, &default_config());
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
        let result = le.find_links(&input, &default_config());
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
        let result = le.find_links(&input, &default_config());
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
        let result = le.find_links(input, &default_config());
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
        let result = le.find_links(input, &default_config());
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
        let result = le.find_links("123<a href=\"http://example.net/\"> link text</a>", &default_config());
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
        let result = le.find_links("\n123<a href=\"http://example.net/\"> link text</a>", &default_config());
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
        let result = le.find_links("Some text <a href=\"some_url\">link text</a> more text.", &default_config());
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
        let result = le.find_links(&input, &default_config());
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
        let input = format!("Foo Bar\n\n[Arbitrary CASE-insensitive reference text]: {link_str}");
        let result = le.find_links(&input, &default_config());
        assert_eq!(0, result.len());
    }

    #[test]
    fn referenced_link_no_tag_only() {
        let le = MarkdownLinkExtractor();
        let input = "[link][reference]";
        let result = le.find_links(input, &default_config());
        assert_eq!(1, result.len());
    }

    // Tests for code block link checking feature
    #[test]
    fn code_block_with_url_not_checked_by_default() {
        let le = MarkdownLinkExtractor();
        let input = "```bash\nwget https://raw.githubusercontent.com/example/file.txt\n```";
        let result = le.find_links(input, &default_config());
        assert!(result.is_empty());
    }

    #[test]
    fn code_block_with_url_checked_when_enabled() {
        let le = MarkdownLinkExtractor();
        let input = "```bash\nwget https://raw.githubusercontent.com/example/file.txt\n```";
        let result = le.find_links(input, &config_with_code_blocks());
        assert_eq!(1, result.len());
        let link = result[0].as_ref().unwrap();
        assert_eq!(link.target, "https://raw.githubusercontent.com/example/file.txt");
    }

    #[test]
    fn inline_code_with_url_checked_when_enabled() {
        let le = MarkdownLinkExtractor();
        let input = "Use `wget https://example.com/file.txt` to download.";
        let result = le.find_links(input, &config_with_code_blocks());
        assert_eq!(1, result.len());
        let link = result[0].as_ref().unwrap();
        assert_eq!(link.target, "https://example.com/file.txt");
    }

    #[test]
    fn code_block_with_multiple_urls() {
        let le = MarkdownLinkExtractor();
        let input = "```\nwget https://example.com/file1.txt\ncurl http://example.org/file2.txt\n```";
        let result = le.find_links(input, &config_with_code_blocks());
        assert_eq!(2, result.len());
        assert_eq!(result[0].as_ref().unwrap().target, "https://example.com/file1.txt");
        assert_eq!(result[1].as_ref().unwrap().target, "http://example.org/file2.txt");
    }

    #[test]
    fn code_block_url_with_special_chars() {
        let le = MarkdownLinkExtractor();
        let input = "```\nhttps://example.com/path?param=value&other=123\n```";
        let result = le.find_links(input, &config_with_code_blocks());
        assert_eq!(1, result.len());
        assert_eq!(result[0].as_ref().unwrap().target, "https://example.com/path?param=value&other=123");
    }

    #[test]
    fn code_block_url_with_parentheses() {
        let le = MarkdownLinkExtractor();
        let input = "```\nSee https://en.wikipedia.org/wiki/Markdown_(markup_language) for details\n```";
        let result = le.find_links(input, &config_with_code_blocks());
        assert_eq!(1, result.len());
        // URL should include the parentheses in the path
        assert_eq!(result[0].as_ref().unwrap().target, "https://en.wikipedia.org/wiki/Markdown_(markup_language)");
    }

    #[test]
    fn code_block_url_ending_with_punctuation() {
        let le = MarkdownLinkExtractor();
        let input = "```\nVisit https://example.com/page. Then do something.\n```";
        let result = le.find_links(input, &config_with_code_blocks());
        assert_eq!(1, result.len());
        // URL should NOT include the trailing period
        assert_eq!(result[0].as_ref().unwrap().target, "https://example.com/page");
    }
}
