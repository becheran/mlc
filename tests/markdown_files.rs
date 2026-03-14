#[cfg(test)]
use mlc::link_extractors::link_extractor::find_links;
use mlc::markup::{MarkupFile, MarkupType};
use mlc::{Config, OptionalConfig};
use std::path::PathBuf;

fn default_config() -> Config {
    Config {
        directory: PathBuf::from("."),
        optional: OptionalConfig::default(),
    }
}

#[test]
fn no_links() {
    let path = "./benches/benchmark/markdown/no_links/no_links.md".to_string();
    let file = MarkupFile {
        path,
        markup_type: MarkupType::Markdown,
    };
    let result = find_links(&file, &default_config());
    assert!(result.is_empty());
}

#[test]
fn some_links() {
    let path = "./benches/benchmark/markdown/many_links/many_links.md".to_string();
    let file = MarkupFile {
        path,
        markup_type: MarkupType::Markdown,
    };
    let result = find_links(&file, &default_config());
    assert_eq!(result.len(), 12);
}
