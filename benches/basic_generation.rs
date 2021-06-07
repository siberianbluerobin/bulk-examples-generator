use bulk_examples_generator::config::*;
use bulk_examples_generator::generate_examples;
use criterion::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Instant;

fn basic_benchmark(c: &mut Criterion) {
    let gen_config: GeneratorConfig = Default::default();
    let mut exe_config: ExecutorConfig = ExecutorConfig::benchmark();
    exe_config.print_stdout = false;
    let grammar = r#"
            // I like Rust!
            language = {"Rust" | "Python" | "Go" | "Java" | "PHP" | "Haskell"}
            one = {"1"}
            daysNumber = {one ~ " day" | !one ~ ASCII_NONZERO_DIGIT ~ " days"}
            sentence = {"I have been programming in " ~ language ~ " for " ~ daysNumber ~ "."}
        "#;

    c.bench_function("Readme grammar example", |b| {
        b.iter(|| {
            generate_examples(
                black_box(grammar.to_string()),
                black_box(100),
                black_box("sentence".to_string()),
                black_box(&gen_config),
                black_box(&exe_config),
            );
        })
    });
}

/// Performance sequential reference 14 - 15 KiB/sec
/// Add Rc to processing stack 30 - 40 KiB/sec
fn throughput_sequential_benchmark_readme_example(c: &mut Criterion) {
    // let config: GeneratorConfig = GeneratorConfig::benchmark();
    let mut exe_config: ExecutorConfig = Default::default();
    exe_config.parallel_mode = false;
    exe_config.print_stdout = false;
    exe_config.return_vec = true;

    let mut gen_config: GeneratorConfig = Default::default();
    gen_config.terminals_limit = Some(500);
    let grammar = r#"
            // https://www.fuzzingbook.org/html/GrammarFuzzer.html
            // EXPR_EBNF_GRAMMAR
            start = {expr}
            // expr = {(term ~ "+" ~ expr) | (term ~ "-" ~ expr) | (expr)} <- left recursive
            expr = {((term ~ "+") | (term ~ "-")) ~ expr{0,1}}
            term = {(factor ~ "*" ~ term) | (factor ~   "/" ~ term) | factor}
            // factor = {(sign1 ~ factor) | ( "(" ~ expr ~ ")" ) | (integer ~ symbol1)} <- left recursive
            // Maybe the probabilities are modified by the refactoring
            factor = { factorA ~ (sign1 ~ factorA){0,1} }
            factorA = {( "(" ~ expr ~ ")" ) | (integer ~ symbol1) }
            sign = {"+" | "-"}
            integer = {digit1}
            digit = {"0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" }
            symbol = {"." ~ integer}
            sign1 = {sign | "" }
            symbol1 = {symbol | "" }
            digit1 = {digit | digit ~ digit1{0,1}}
        "#;

    let mut group = c.benchmark_group("throughput_benchmark_readme_example");
    group.sample_size(10);
    group.bench_function("Readme grammar example", |b| {
        b.iter(|| {
            let it = Instant::now();
            let s = generate_examples(
                black_box(grammar.to_string()),
                black_box(20),
                black_box("start".to_string()),
                black_box(&gen_config),
                black_box(&exe_config),
            );
            let elapsed = (Instant::now() - it).as_secs_f64();
            let sum_len = s
                .iter()
                .map(|x| x.as_ref().unwrap())
                .fold(String::new(), |acc, s| acc + &s)
                .len();
            let bytes_per_sec = sum_len as f64 / elapsed;
            // println!("{:?}", s);
            // println!("{}", sum_len);
            // print!("MiB/sec: {:12.4}\n", bytes_per_sec / 1024. / 1024.);
            print!("KiB/sec: {:12.8}\n", bytes_per_sec / 1024.);
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    basic_benchmark,
    throughput_sequential_benchmark_readme_example
);
criterion_main!(benches);
