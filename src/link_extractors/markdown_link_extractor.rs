use crate::link_extractors::link_extractor::LinkExtractor;
use crate::link_extractors::link_extractor::MarkupLink;

extern crate comrak;
use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};
use std::str;

pub struct MarkdownLinkExtractor();

impl LinkExtractor for MarkdownLinkExtractor {
    fn find_links(&self, text: &str) -> Vec<MarkupLink> {
        let arena = Arena::new();

        // TODO: OPTIONS
        let root = parse_document(&arena, text, &ComrakOptions::default());

        fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &mut F)
        where
            F: FnMut(&'a AstNode<'a>),
        {
            f(node);
            for c in node.children() {
                iter_nodes(c, f);
            }
        }

        let mut result: Vec<MarkupLink> = Vec::new();
        iter_nodes(root, &mut |node| {
            let data = node.data.borrow();
            println!("{:?}", data);
            match &data.value {
                NodeValue::Link(ref link) => {
                    // TODO ERR IF NOT UTF-8??
                    let target_url = str::from_utf8(&link.url).unwrap().to_string();
                    result.push(MarkupLink {
                        column: 0,
                        line: data.start_line as usize,
                        source: String::new(),
                        target: target_url,
                    });
                }
                NodeValue::Image(ref link) => {
                    // TODO ERR IF NOT UTF-8??
                    let target_url = str::from_utf8(&link.url).unwrap().to_string();
                    result.push(MarkupLink {
                        column: 0,
                        line: data.start_line as usize,
                        source: String::new(),
                        target: target_url,
                    });
                }
                _ => (),
            }
        });

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
    fn commented_link() {
        let le = MarkdownLinkExtractor();
        let input = "]This is not a () <!--[link](link)-->.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn nested_links() {
        let le = MarkdownLinkExtractor();
        let input = "\nfoo\n\n[![](http://meritbadge.herokuapp.com/mlc)](https://crates.io/crates/mlc)";
        let result = le.find_links(&input);
        let img = MarkupLink {
            target: "http://meritbadge.herokuapp.com/mlc".to_string(),
            line: 1,
            column: 5,
            source: "".to_string(),
        };
        let link = MarkupLink {
            target: "https://crates.io/crates/mlc".to_string(),
            line: 1,
            column: 43,
            source: "".to_string(),
        };
        assert_eq!(vec![img, link], result);
    }

    #[test]
    fn link_escaped() {
        let le = MarkdownLinkExtractor();
        let input = "This is not a \\[link\\](random_link).";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn link_in_headline() {
        let le = MarkdownLinkExtractor();
        let input = "  # This is a [link](http://example.net/).";
        let result = le.find_links(&input);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn no_link_colon() {
        let le = MarkdownLinkExtractor();
        let input = "This is not a [link]:bla.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn inline_code() {
        let le = MarkdownLinkExtractor();
        let input = " `[code](http://example.net/)`, no link!.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn link_near_inline_code() {
        let le = MarkdownLinkExtractor();
        let input = " `bug` [code](http://example.net/), link!.";
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column: 14,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_very_near_inline_code() {
        let le = MarkdownLinkExtractor();
        let input = "`bug`[code](http://example.net/)";
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column: 13,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn code_block() {
        let le = MarkdownLinkExtractor();
        let input = " ``` js\n[code](http://example.net/)```, no link!.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn html_code_block() {
        let le = MarkdownLinkExtractor();
        let input = "<script>\n[code](http://example.net/)</script>, no link!.";
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn escaped_code_block() {
        let le = MarkdownLinkExtractor();
        let input = "   klsdjf \\`[escape](http://example.net/)\\`, no link!.";
        let result = le.find_links(&input);
        let expected = MarkupLink {
            target: "http://example.net/".to_string(),
            line: 1,
            column: 21,
            source: "".to_string(),
        };
        assert_eq!(vec![expected], result);
    }

    #[test]
    fn link_in_code_block() {
        let le = MarkdownLinkExtractor();
        let input = "```\n[only code](http://example.net/)\n```.";
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
            column: 29,
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
            column: 12,
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
            column: 20,
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
            column,
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
