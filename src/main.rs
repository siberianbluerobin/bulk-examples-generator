use bulk_examples_generator::compile_grammar;
use bulk_examples_generator::config::GeneratorConfig;
use bulk_examples_generator::parallel_generate_examples;
use bulk_examples_generator::parallel_generate_save_examples;

use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
/// Generate massive amounts of random examples based in a PEST grammar, it can be used like a grammar fuzzer
///
/// Examples of use:
///
/// bulk-examples-generator --grammar my-grammar.pest --quantity 5 --start-rule myrule --out-type stdout
///
/// Shortened
///
/// bulk-examples-generator -g my-grammar.pest -q 5 -s myrule -o stdout
#[structopt(name = "bulk-examples-generator")]
pub struct Opt {
    /// Path of grammar for generate examples
    #[structopt(short, long, parse(from_os_str))]
    pub grammar: PathBuf,

    /// Quantity of examples to generate
    #[structopt(short, long)]
    pub quantity: u32,

    /// Rule to start generation of examples
    #[structopt(short, long)]
    pub start_rule: String,

    /// Where to write the examples: one of debug, stdout, folder
    ///
    /// debug: Print results in stdout (vec form) for debugging purposes
    /// stdout: Print results in stdout
    /// folder: Create one file for each example (use template_name for personalize the filename)
    ///
    #[structopt(short, long, verbatim_doc_comment)]
    pub out_type: String,

    /// Used when the out-type is stdout
    ///
    /// Print "Example #n generated:" before print the example
    #[structopt(long)]
    pub print_progress: bool,

    // TODO: is necessary implement this?
    // /// file: Save all examples in a single file
    // #[structopt(required_if("out_type", "file"), parse(from_os_str))]
    // pub output_file: Option<PathBuf>,
    /// Output folder to save the examples
    #[structopt(long, required_if("out_type", "file"), parse(from_os_str))]
    pub output_folder: Option<PathBuf>,

    /// Name of the files, e.g. html-test-{}.html, {} will be used for enumerating the example
    #[structopt(short, long, default_value = "example-{}.txt")]
    pub template_name: String,

    /// Config file for generate elements, for more details pleaser refer to README
    /// Default config available in src/config/default.toml
    #[structopt(short, long, parse(from_os_str))]
    pub config_file: Option<PathBuf>,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let mut config: GeneratorConfig = Default::default();
    if let Some(config_file) = &opt.config_file {
        config = GeneratorConfig::new(config_file.to_str().unwrap()).unwrap();
    }

    // Load grammar file
    let mut grammar_string = String::new();
    let mut f = File::open(&opt.grammar)?;
    f.read_to_string(&mut grammar_string)?;

    if opt.out_type == "debug" {
        // Print input parameters
        println!("{:?}", &opt);

        // Print grammar
        let g = compile_grammar(grammar_string.clone());
        println!("{:?}", g);

        // Generating examples and just print the vector
        let results = parallel_generate_examples(
            grammar_string,
            opt.quantity,
            opt.start_rule,
            &config,
            true,
            false,
        );
        println!("{:?}", results);
    } else if opt.out_type == "stdout" {
        parallel_generate_examples(
            grammar_string,
            opt.quantity,
            opt.start_rule,
            &config,
            opt.print_progress,
            true,
        );
    } else {
        // Generate examples and save in a defined path
        parallel_generate_save_examples(
            grammar_string,
            opt.quantity,
            opt.start_rule,
            opt.output_folder.unwrap(),
            opt.template_name,
            &config,
        );
    }

    Ok(())
}
