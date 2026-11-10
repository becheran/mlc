use criterion::{Criterion, criterion_group, criterion_main};
use mlc::markup::MarkupType;
use mlc::{Config, OptionalConfig};
use std::fs;

async fn end_to_end_benchmark() {
    let config = Config {
        directory: fs::canonicalize("./benches/benchmark/markdown/ignore_me_dir").unwrap(),
        optional: OptionalConfig {
            markup_types: Some(vec![MarkupType::Markdown]),
            ..Default::default()
        },
    };
    let _ = mlc::run(&config).await;
}

/// Many local links across all benchmark files, offline so no network noise
async fn many_links_benchmark() {
    let config = Config {
        directory: fs::canonicalize("./benches/benchmark").unwrap(),
        optional: OptionalConfig {
            markup_types: Some(vec![MarkupType::Markdown, MarkupType::Html]),
            offline: Some(true),
            ..Default::default()
        },
    };
    let _ = mlc::run(&config).await;
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("End to end benchmark", |b| {
        b.iter(|| rt.block_on(end_to_end_benchmark()))
    });
    c.bench_function("Many links offline", |b| {
        b.iter(|| rt.block_on(many_links_benchmark()))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
