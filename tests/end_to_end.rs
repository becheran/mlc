#[cfg(test)]
use mlc::Config;
use mlc::markup::MarkupType;
use mlc::logger;

#[test]
fn end_to_end() {
    let config = Config {
        folder: String::from("./tests/benchmark"),
        log_level: logger::LogLevel::Debug,
        markup_types: vec![MarkupType::Markdown],
    };
    let _ = mlc::run(&config);
}

#[bench]
fn bench_xor_1000_ints(b: &mut Bencher) {
    b.iter(|| {
        (0..1000).fold(0, |old, new| old ^ new);
    });
}