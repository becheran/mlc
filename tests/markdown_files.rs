#[cfg(test)]
use mlc::link_extractor;
use mlc::markup::{MarkupFile, MarkupType};


#[test]
fn no_links() {
    let path = "./benches/benchmark/no_links/no_links.md".to_string();
    let file = MarkupFile {
        path,
        markup_type: MarkupType::Markdown,
    };
    let result = link_extractor::find_links(&file);
    assert!(result.is_empty());
}

#[test]
fn some_links() {
    let path = "./benches/benchmark/many_links/many_links.md".to_string();
    let file = MarkupFile {
        path,
        markup_type: MarkupType::Markdown,
    };
    let result = link_extractor::find_links(&file);
    assert_eq!(result.len(), 10);
}
