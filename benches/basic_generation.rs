use bulk_examples_generator::config::GeneratorConfig;
use bulk_examples_generator::parallel_generate_examples;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn basic_benchmark(c: &mut Criterion) {
    let config: GeneratorConfig = GeneratorConfig::benchmark();
    let grammar = r#"
            // I like Rust!
            language = {"Rust" | "Python" | "Go" | "Java" | "PHP" | "Haskell"}
            one = {"1"}
            daysNumber = {one ~ " day" | !one ~ ASCII_NONZERO_DIGIT ~ " days"}
            sentence = {"I have been programming in " ~ language ~ " for " ~ daysNumber ~ "."}
        "#;

    c.bench_function("Readme grammar example", |b| {
        b.iter(|| {
            parallel_generate_examples(
                black_box(grammar.to_string()),
                black_box(100),
                black_box("sentence".to_string()),
                black_box(&config),
                black_box(false),
                black_box(true),
            )
        })
    });
}

criterion_group!(benches, basic_benchmark);
criterion_main!(benches);
