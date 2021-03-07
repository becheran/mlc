use crate::link_extractors::link_extractor::LinkExtractor;
use crate::link_extractors::link_extractor::MarkupLink;

pub struct MarkdownLinkExtractor();

fn skip_whitespace(vector: &Vec<char>, pos: &mut usize) {
    while *pos < vector.len() && vector[*pos].is_whitespace() {
        *pos += 1;
    }
}

/// Advance the `pos` index in the vector until reaching a character that
/// matches the character at the index `pos` began at.
///
/// # Examples
///
/// ```ignore
/// let vector = vec!['(', 'h', 'e', 'l', 'l', 'o', ' ', '(', 'w', ')', 'o', 'r', 'l', 'd', ')'];
/// let pos = 0;
///
/// let matching_char = forward_until_matching(&vector, &mut pos);
/// assert_eq!(pos, 14);
/// assert_eq!(matching_char, ')');
/// ```
fn forward_until_matching(vector: &Vec<char>, pos: &mut usize) -> bool {
    let start_char = vector.get(*pos);
    let matching_char = match start_char {
        Some(&'`') => Some(&'`'),
        Some(&'(') => Some(&')'),
        Some(&'[') => Some(&']'),
        _ => panic!(
            "no matching arm for char {:?} in forward_until_matching",
            start_char
        ),
    };

    // iterate through the chars until we've reached a char that matches
    // the start_char we began at
    let mut num_unmatched_start_chars = 1;
    while vector.get(*pos).is_some() && num_unmatched_start_chars != 0 {
        *pos += 1;

        // keep track of any new start_chars we find so we know when we've found
        // a char that matches our first start_char
        if vector.get(*pos) == start_char {
            num_unmatched_start_chars += 1;
        }
        if vector.get(*pos) == matching_char {
            num_unmatched_start_chars -= 1;
        }
    }
    vector.get(*pos).is_some()
}

impl LinkExtractor for MarkdownLinkExtractor {
    fn find_links(&self, text: &str) -> Vec<MarkupLink> {
        let mut result: Vec<MarkupLink> = Vec::new();
        let mut link_tags: Vec<String> = Vec::new();
        let mut reference_link_tags: Vec<String> = Vec::new();
        let mut is_code_block = false;
        let mut is_comment = false;
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
            } else if line_chars.get(column) == Some(&'<')
                && line_chars.get(column + 1) == Some(&'/')
                && line_chars.get(column + 2) == Some(&'s')
                && line_chars.get(column + 3) == Some(&'c')
                && line_chars.get(column + 4) == Some(&'r')
                && line_chars.get(column + 5) == Some(&'i')
                && line_chars.get(column + 6) == Some(&'p')
                && line_chars.get(column + 7) == Some(&'t')
                && line_chars.get(column + 8) == Some(&'>')
            {
                is_code_block = false;
                column += 8;
            } else if line_chars.get(column) == Some(&'-')
                && line_chars.get(column + 1) == Some(&'-')
                && line_chars.get(column + 2) == Some(&'>')
            {
                is_comment = false;
                column += 2;
            }

            if is_code_block || is_comment {
                continue;
            }

            while column < line_chars.len() {
                if is_comment && line_chars[column] != '-' {
                    column += 1;
                    continue;
                }
                match line_chars[column] {
                    '`' => {
                        forward_until_matching(&line_chars, &mut column);
                    }
                    '\\' => {
                        column += 1; // Escape next character
                    }
                    '[' => {
                        let square_bracket_start = column;
                        if forward_until_matching(&line_chars, &mut column) {
                            let square_bracket_close = column;

                            // recurse on line of text within the square brackets to find any
                            // nested links contained within it
                            let nested_text = (&line_chars
                                [square_bracket_start + 1..square_bracket_close])
                                .iter()
                                .collect::<String>();
                            let mut nested_links = self.find_links(&nested_text);

                            // at this point any MarkupLinks in nested_links have an
                            // incorrect column field, and to correct them we need to
                            // account for the offset of nested_text within line_chars
                            for mut link in nested_links.iter_mut() {
                                link.column += square_bracket_start + 1;
                            }
                            result.append(&mut nested_links);

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
                                    if forward_until_matching(&line_chars, &mut column) {
                                        let link = (&line_chars[start_idx..column])
                                            .iter()
                                            .collect::<String>();
                                        // Take first split because of possible title tag
                                        let mut spl = link.split_whitespace();
                                        let link = spl.next().unwrap_or("");
                                        debug!("Extract link link in format []() {:?}", link);
                                        result.push(MarkupLink {
                                            column: bracket_start + 1,
                                            line: line + 1,
                                            target: link.to_string(),
                                            source: "".to_string(),
                                        });
                                    }
                                }
                                Some(&'[') => {
                                    if forward_until_matching(&line_chars, &mut column) {
                                        let reference_link = (&line_chars[start_idx..column])
                                            .iter()
                                            .collect::<String>();
                                        debug!("Extract reference link {:?}", reference_link);
                                        link_tags.push(reference_link.to_lowercase());
                                    }
                                }
                                Some(&':') => {
                                    if line_chars[..square_bracket_start]
                                        .iter()
                                        .any(|c| !c.is_whitespace())
                                    {
                                        continue;
                                    }
                                    column += 1;
                                    skip_whitespace(&line_chars, &mut column);
                                    let start_idx = column;
                                    while column < line_chars.len()
                                        && !line_chars[column].is_whitespace()
                                    {
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
                        match line_chars.get(column) {
                            Some(&'a') | Some(&'A') => {
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
                                        let link = (&line_chars[start_idx..column])
                                            .iter()
                                            .collect::<String>();
                                        result.push(MarkupLink {
                                            column: link_column + 1,
                                            line: line + 1,
                                            target: link.to_string(),
                                            source: "".to_string(),
                                        });
                                    }
                                }
                            }
                            Some(&'s') | Some(&'S') => {
                                if line_chars.get(column + 1) == Some(&'c')
                                    && line_chars.get(column + 2) == Some(&'r')
                                    && line_chars.get(column + 3) == Some(&'i')
                                    && line_chars.get(column + 4) == Some(&'p')
                                    && line_chars.get(column + 5) == Some(&'t')
                                {
                                    column += 5;
                                    is_code_block = true;
                                }
                            }
                            Some(&'!') => {
                                if line_chars.get(column + 1) == Some(&'-')
                                    && line_chars.get(column + 2) == Some(&'-')
                                {
                                    column += 2;
                                    is_comment = true;
                                }
                            }
                            Some(_) | None => {}
                        }
                    }
                    '-' => {
                        if line_chars.get(column + 1) == Some(&'-')
                            && line_chars.get(column + 2) == Some(&'>')
                        {
                            is_comment = false;
                            column += 2;
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
    fn commented_link() {
        let le = MarkdownLinkExtractor();
        let input = "]This is not a () <!--[link](link)-->.";
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
    fn no_link_colon() {
        let le = MarkdownLinkExtractor();
        let input = format!("This is not a [link]:bla.");
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn inline_code() {
        let le = MarkdownLinkExtractor();
        let input = format!(" `[code](http://example.net/)`, no link!.");
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn code_block() {
        let le = MarkdownLinkExtractor();
        let input = format!(" ``` js\n[code](http://example.net/)```, no link!.");
        let result = le.find_links(&input);
        assert!(result.is_empty());
    }

    #[test]
    fn html_code_block() {
        let le = MarkdownLinkExtractor();
        let input = format!("<script>\n[code](http://example.net/)</script>, no link!.");
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
            column: 21,
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
