#[cfg(test)]
#[macro_use]
extern crate criterion;

use criterion::Criterion;
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
    mlc::run(&config).await.unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("End to end benchmark", |b| b.iter(end_to_end_benchmark));
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}
criterion_main!(benches);
