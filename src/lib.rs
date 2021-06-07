#![doc(html_root_url = "https://docs.rs/bulk_examples_generator/0.1.0")]
//! # Usage
//!
//! bulk_examples_generator come in two flavors: binary or crate (library)
//!
//! For a basic/medium/advanced usage of the binary/library, please see the [Readme](https://github.com/siberianbluerobin/bulk-examples-generator).
//!
//! ## Frequently Asked Questions? (FAQ)
//!
//! See the [Readme](https://github.com/siberianbluerobin/bulk-examples-generator).
//!
//! ## I just want to see how this code works
//!
//! Please see first the Readme. Well if you really want to see the code, go ahead
//!
use aho_corasick::AhoCorasick;
use indicatif::{ProgressBar, ProgressStyle};
use pest::error::{Error, ErrorVariant, InputLocation};
use pest_meta::ast::Rule as AstRule;
use pest_meta::parser::{self, Rule};
use pest_meta::{optimizer, validator};
use pest_vm::Vm;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub mod config;
mod generator;

// Re-exports
pub use pest;
pub use pest_meta;

use crate::config::*;
use crate::generator::*;

/// Compile a grammar string and creates a `HashMap` with rules found as keys and their components (AST) as entries
///
/// In this step, the grammar is validated with the pest reference grammar, and the built-in rules are replaced for
/// their equivalents
/// ```
/// use bulk_examples_generator::compile_grammar;
///
/// // Grammar string
/// let mut grammar = r#"
///         language = {"Rust" | "Python" | "Go" | "Java" | "PHP" | "Haskell"}
///         one = {"1"}
///         daysNumber = {one ~ " day" | !one ~ ASCII_NONZERO_DIGIT ~ " days"}
///         sentence = {"I have been programming in " ~ language ~ " for " ~ daysNumber ~ "."}
///     "#;
///
/// // Generate the ast
/// let grammar_ast = compile_grammar(grammar.to_string());
///
/// println!("{:?}", grammar_ast);
/// ```
pub fn compile_grammar(grammar: String) -> Result<Grammar, Vec<HashMap<String, String>>> {
    // Replace builtin pest rules for their equivalents
    let grammar = replace_builtin_rules(&grammar).unwrap();

    // Print grammar after replaces builtin rules
    // println!("{}", grammar.clone());

    let result = parser::parse(Rule::grammar_rules, &grammar).map_err(|error| {
        error.renamed_rules(|rule| match *rule {
            Rule::grammar_rule => "rule".to_owned(),
            Rule::_push => "push".to_owned(),
            Rule::assignment_operator => "`=`".to_owned(),
            Rule::silent_modifier => "`_`".to_owned(),
            Rule::atomic_modifier => "`@`".to_owned(),
            Rule::compound_atomic_modifier => "`$`".to_owned(),
            Rule::non_atomic_modifier => "`!`".to_owned(),
            Rule::opening_brace => "`{`".to_owned(),
            Rule::closing_brace => "`}`".to_owned(),
            Rule::opening_paren => "`(`".to_owned(),
            Rule::positive_predicate_operator => "`&`".to_owned(),
            Rule::negative_predicate_operator => "`!`".to_owned(),
            Rule::sequence_operator => "`&`".to_owned(),
            Rule::choice_operator => "`|`".to_owned(),
            Rule::optional_operator => "`?`".to_owned(),
            Rule::repeat_operator => "`*`".to_owned(),
            Rule::repeat_once_operator => "`+`".to_owned(),
            Rule::comma => "`,`".to_owned(),
            Rule::closing_paren => "`)`".to_owned(),
            Rule::quote => "`\"`".to_owned(),
            Rule::insensitive_string => "`^`".to_owned(),
            Rule::range_operator => "`..`".to_owned(),
            Rule::single_quote => "`'`".to_owned(),
            other_rule => format!("{:?}", other_rule),
        })
    });

    let pairs = match result {
        Ok(pairs) => pairs,
        Err(error) => {
            // add_rules_to_select(vec![]);
            return Err(vec![convert_error(error, &grammar)]);
        }
    };

    if let Err(errors) = validator::validate_pairs(pairs.clone()) {
        // add_rules_to_select(vec![]);
        return Err(errors
            .into_iter()
            .map(|e| convert_error(e, &grammar))
            .collect());
    }

    let ast = match parser::consume_rules(pairs) {
        Ok(ast) => ast,
        Err(errors) => {
            // add_rules_to_select(vec![]);
            return Err(errors
                .into_iter()
                .map(|e| convert_error(e, &grammar))
                .collect());
        }
    };

    let hashmap_ast_rules: HashMap<String, AstRule> = ast
        .iter()
        .map(|rule| (rule.name.to_string(), rule.clone()))
        .collect();

    Ok(Grammar {
        rules: hashmap_ast_rules,
    })
}

/// # Generate examples
/// Generate a number of examples with the grammar,start rule and config provided
///
/// ```
/// use bulk_examples_generator::config::*;
/// use bulk_examples_generator::generate_examples;
///
/// // Default configuration for the generator
/// let mut gen_config: GeneratorConfig = Default::default();
/// let mut exe_config: ExecutorConfig = Default::default();
/// // exe_config.print_progress
/// // exe_config.print_stdout
///
/// // Grammar string
/// let mut grammar = r#"
///         language = {"Rust" | "Python" | "Go" | "Java" | "PHP" | "Haskell"}
///         one = {"1"}
///         daysNumber = {one ~ " day" | !one ~ ASCII_NONZERO_DIGIT ~ " days"}
///         sentence = {"I have been programming in " ~ language ~ " for " ~ daysNumber ~ "."}
///     "#;
///
/// // Generate the examples
/// let results = generate_examples(
///             grammar.to_string(),        // The grammar
///             5,                          // Quantity of examples
///             "sentence".to_string(),    // Start rule
///             &gen_config,                    // Config of the generator
///             &exe_config,                      // Print progress
///         );
///
/// println!("{:?}", results);
/// ```
/// # Saving examples in a folder
/// Generate and save a number of examples with the grammar,start rule and config provided
///
/// ```ignore
/// # // This doc_test is ignored because have side effects (the files)
/// use std::path::PathBuf;
/// use bulk_examples_generator::config::*;
/// use bulk_examples_generator::generate_save_examples;
///
/// // Default configuration for the generator
/// let mut config: GeneratorConfig = Default::default();
///
/// // Config folder
/// let mut exe_config: ExecutorConfig = Default::default();
/// // Folder to save the examples
/// let path = PathBuf::from(r"C:\my-folder");
/// exe_config.print_folder = Some(("hola-{}.txt".to_string(), path))
///
/// // Grammar string
/// let mut grammar = r#"
///         language = {"Rust" | "Python" | "Go" | "Java" | "PHP" | "Haskell"}
///         one = {"1"}
///         daysNumber = {one ~ " day" | !one ~ ASCII_NONZERO_DIGIT ~ " days"}
///         sentence = {"I have been programming in " ~ language ~ " for " ~ daysNumber ~ "."}
///     "#;
///
/// // Generate and save the examples
/// let results = generate_examples(
///             grammar.to_string(),       // The grammar
///             5,                         // Quantity of examples
///             "sentence".to_string(),   // Start rule
///             &gen_config,                   // Config of the generator
///             &exe_config,                   // Config of the executor
///         );
///
/// ```
pub fn generate_examples(
    grammar_string: String,
    quantity: u32,
    start: String,
    generator_config: &GeneratorConfig,
    executor_config: &ExecutorConfig,
) -> Vec<Result<String, String>> {
    let input_data = InputData::new(grammar_string);
    if executor_config.parallel_mode {
        return parallel_generate_examples(
            input_data,
            quantity,
            start,
            generator_config,
            executor_config,
        );
    } else {
        return sequential_generate_examples(
            input_data,
            quantity,
            start,
            generator_config,
            executor_config,
        );
    }
}

fn parallel_generate_examples(
    input_grammar: InputData,
    quantity: u32,
    start: String,
    generator_config: &GeneratorConfig,
    executor_config: &ExecutorConfig,
) -> Vec<Result<String, String>> {
    let vec = Arc::new(Mutex::new(vec![]));

    // Create the progress bar
    let progress_bar = ProgressBar::new(quantity.into());
    if executor_config.print_progress_bar {
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "[{elapsed_precise}] {wide_bar} {pos:>3}/{len:3} {msg} {percent}% {eta_precise}",
                )
                .progress_chars("██░"),
        );

        // Force the initial paint
        progress_bar.tick();
    }

    (1..quantity + 1).into_par_iter().for_each(|i| {
        let r = generator::generate_example(input_grammar.clone(), start.clone(), generator_config);
        if executor_config.print_progress_bar {
            progress_bar.inc(1);
        }
        if executor_config.print_progress_text {
            println!("Example #{} generated:\r\n{}", i, r.as_ref().unwrap());
        }
        if executor_config.print_stdout {
            println!("{}", r.as_ref().unwrap());
        }

        if executor_config.return_vec {
            vec.lock().unwrap().push(r.clone())
        }

        if let Some((name_format, folder_path)) = &executor_config.print_folder {
            match r {
                Ok(example) => {
                    let new_path = folder_path.join(name_format.replace("{}", &i.to_string()));
                    // println!("for {:?}", new_path);
                    // Save the file
                    let mut f = File::create(new_path).expect("Unable to create file");
                    f.write_all(example.as_bytes())
                        .expect(&format!("Unable to write data, example {}", i));
                }
                Err(error) => {
                    println!("{}", error);
                }
            }
        }
    });

    if executor_config.print_progress_bar {
        progress_bar.finish();
    }

    Arc::try_unwrap(vec).unwrap().into_inner().unwrap()
}

fn sequential_generate_examples(
    input_grammar: InputData,
    quantity: u32,
    start: String,
    generator_config: &GeneratorConfig,
    executor_config: &ExecutorConfig,
) -> Vec<Result<String, String>> {
    let mut vec = vec![];

    // Create progress bar
    let progress_bar = ProgressBar::new(quantity.into());
    if executor_config.print_progress_bar {
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "[{elapsed_precise}] {wide_bar} {pos:>3}/{len:3} {msg} {percent}% {eta_precise}",
                )
                .progress_chars("██░"),
        );

        // Force the initial paint
        progress_bar.tick();
    }

    for i in 1..quantity + 1 {
        // Generate example
        let r = generator::generate_example(input_grammar.clone(), start.clone(), generator_config);
        if executor_config.print_progress_bar {
            progress_bar.inc(1);
        }
        if executor_config.print_progress_text {
            println!("Example #{} generated:\r\n{}", i, r.as_ref().unwrap());
        }
        if executor_config.print_stdout {
            println!("{}", r.as_ref().unwrap());
        }

        if executor_config.return_vec {
            vec.push(r.clone())
        }

        if let Some((name_format, folder_path)) = &executor_config.print_folder {
            match r {
                Ok(example) => {
                    let new_path = folder_path.join(name_format.replace("{}", &i.to_string()));
                    // println!("for {:?}", new_path);
                    // Save the file
                    let mut f = File::create(new_path).expect("Unable to create file");
                    f.write_all(example.as_bytes())
                        .expect(&format!("Unable to write data, example {}", i));
                }
                Err(error) => {
                    println!("{}", error);
                }
            }
        }
    }

    if executor_config.print_progress_bar {
        progress_bar.finish();
    }

    vec
}

// Parsea `input` usando la gramática `grammar`, iniciando el parseo desde `rule`
// retorna Ok si es exitoso el parseo, Err si no es posible parsear
// Es usado en términos generales como shorcut en los tests para validar si una cadena generada, puede ser parseada por la misma gramatica que la genero
/// Parse input with the provided grammar and start rule; returns `Ok` if the parse is successful, `Err` otherwise
///
/// It's used for validate the examples generated with the original grammar
///
/// ```
/// use bulk_examples_generator::config::*;
/// use bulk_examples_generator::{compile_grammar, parse_input, generate_examples};
///
/// // Default configuration for the generator
/// let mut gen_config: GeneratorConfig = Default::default();
/// let mut exe_config: ExecutorConfig = Default::default();
/// exe_config.return_vec = true;
///
/// // Grammar string
/// let mut grammar = r#"
///         language = {"Rust" | "Python" | "Go" | "Java" | "PHP" | "Haskell"}
///         one = {"1"}
///         daysNumber = {one ~ " day" | !one ~ ASCII_NONZERO_DIGIT ~ " days"}
///         sentence = {"I have been programming in " ~ language ~ " for " ~ daysNumber ~ "."}
///     "#;
///
/// // Generate the ast
/// let grammar_ast = compile_grammar(grammar.to_string());
///
/// // Generate the examples
/// let results = generate_examples(
///             grammar.to_string(),        // The grammar
///             1,                          // Quantity of examples
///             "sentences".to_string(),    // Start rule
///             &gen_config,                    // Config of the generator
///             &exe_config,                    // Config of the executor
///         );
///
/// let one_example = results[0].as_ref().unwrap();
///
/// // Validate the generated example with the grammar
/// let validate = parse_input(grammar_ast.unwrap(), "sentence".to_string(), one_example.to_string());
///
/// println!("{:?}", validate);
/// ```
///
pub fn parse_input<'a>(grammar: Grammar, rule: String, input: String) -> Result<(), String> {
    // Es necesario entregar una copia entera de las reglas al vm
    let vm = Vm::new(optimizer::optimize(
        grammar.rules.values().map(|r| r.clone()).collect(),
    ));

    parse_input_with_vm(vm, rule, input)
}

/// Parsea `input` usando la máquina `Vm`, iniciando el parseo desde `rule`
/// retorna Ok si es exitoso el parseo, Err si no es posible parsear
fn parse_input_with_vm<'a>(vm: Vm, rule: String, input: String) -> Result<(), String> {
    match vm.parse(&rule, &input) {
        Ok(_pairs) => {
            // let lines: Vec<_> = pairs.map(|pair| format_pair(pair, 0, true)).collect();
            // let lines = lines.join("\n");

            // output.set_value(&format!("{}", lines));
            Ok(())
        }
        Err(error) => {
            // output.set_value(&format!("{}", error.renamed_rules(|r| r.to_string())))
            // FIXME: Eliminar el string para usar un tipo de error más "Rustacean"
            Err(format!("{}", error.renamed_rules(|r| r.to_string())))
        }
    }
    // }
}

fn convert_error(error: Error<Rule>, grammar: &str) -> HashMap<String, String> {
    let message = match error.variant {
        ErrorVariant::CustomError { message } => message,
        _ => unreachable!(),
    };

    match error.location {
        InputLocation::Pos(pos) => {
            let mut map = HashMap::new();

            map.insert("from".to_owned(), line_col(pos, grammar));
            map.insert("to".to_owned(), line_col(pos, grammar));
            map.insert("message".to_owned(), format!("{}", message));

            map
        }
        InputLocation::Span((start, end)) => {
            let mut map = HashMap::new();

            map.insert("from".to_owned(), line_col(start, grammar));
            map.insert("to".to_owned(), line_col(end, grammar));
            map.insert("message".to_owned(), format!("{}", message));

            map
        }
    }
}

fn line_col(pos: usize, input: &str) -> String {
    let (line, col) = {
        let mut pos = pos;
        // Position's pos is always a UTF-8 border.
        let slice = &input[..pos];
        let mut chars = slice.chars().peekable();

        let mut line_col = (1, 1);

        while pos != 0 {
            match chars.next() {
                Some('\r') => {
                    if let Some(&'\n') = chars.peek() {
                        chars.next();

                        if pos == 1 {
                            pos -= 1;
                        } else {
                            pos -= 2;
                        }

                        line_col = (line_col.0 + 1, 1);
                    } else {
                        pos -= 1;
                        line_col = (line_col.0, line_col.1 + 1);
                    }
                }
                Some('\n') => {
                    pos -= 1;
                    line_col = (line_col.0 + 1, 1);
                }
                Some(c) => {
                    pos -= c.len_utf8();
                    line_col = (line_col.0, line_col.1 + 1);
                }
                None => unreachable!(),
            }
        }

        line_col
    };

    format!("({}, {})", line - 1, col - 1)
}

/// Replace builtin pest rules for their equivalents
///
/// For example in a grammar like this:
/// ```text
/// small_number = ASCII_DIGIT{3}
/// ```
///
/// the replaced equivalent will be this:
/// ASCII_DIGIT
/// ```text
/// small_number = ('0'..'9'){3}
/// ```
///
/// **Note:** currently only the ASCII rules are replaced
///
/// For the list of equivalent rules see https://pest.rs/book/grammars/built-ins.html
fn replace_builtin_rules(grammar: &String) -> Result<String, std::io::Error> {
    //TODO: Add the Unicode rules from https://pest.rs/book/grammars/built-ins.html

    let patterns = &[
        "ANY",
        "ASCII_DIGIT",
        "ASCII_NONZERO_DIGIT",
        "ASCII_BIN_DIGIT",
        "ASCII_OCT_DIGIT",
        "ASCII_HEX_DIGIT",
        "ASCII_ALPHA_LOWER",
        "ASCII_ALPHA_UPPER",
        "ASCII_ALPHANUMERIC",
        "NEWLINE",
    ];

    // Parentheses are kept to facilitate things like ASCII_ALPHA{1,5}
    let replace_with = &[
        "('\u{00}'..'\u{10FFFF}')",
        "('0'..'9')",
        "('1'..'9')",
        "('0'..'1')",
        "('0'..'7')",
        "('0'..'9' | 'a'..'f' | 'A'..'F')",
        "('a'..'z')",
        "('A'..'Z')",
        "('0'..'9' | 'a'..'z' | 'A'..'Z')",
        r#"("\n" | "\r\n" | "\r")"#,
    ];

    // Replace all strings in a single pass
    let mut wtr = vec![];
    let ac = AhoCorasick::new(patterns);
    ac.stream_replace_all(grammar.as_bytes(), &mut wtr, replace_with)?;

    // println!("{:?}", wtr);
    let mut s = match String::from_utf8(wtr) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    // ASCII_ALPHA it is replaced last because it has conflict with ASCII_ALPHA_LOWER y ASCII_ALPHA_UPPER
    // because the word "ASCII_ALPHA" is shorter
    s = s.replace("ASCII_ALPHA", "('a'..'z' | 'A'..'Z')");

    // println!("result: {}", s);
    Ok(s)
}
