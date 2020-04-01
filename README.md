# Bulk Examples Generator

Bulk Examples Generator is a tool created in Rust for create dozens/hundreds/thousands/millions of random examples based on a pest grammar (PEG). It can be used for generate string/structured data for training models in AI, or like a grammar fuzzer for find bugs.

# Table of Contents

- [Summary](#summary)
- [Binary usage](#binary-usage)
  - [Requirements](#requirements)
  - [Getting started](#getting-started)
  - [Usage](#usage)
  - [Additional functionalities](#additional-functionalities)
    - [Blacklist generation](#blacklist-generation)
  - [Config file](#config-file)
  - [Command line options](#command-line-options)
  - [Benchmarks](#benchmarks)
- [Crate usage](#crate-usage)
  - [Getting started](#getting-started)
  - [Example](#example)
    - [Config](#config)
    - [Available functions](#available-functions)
  - [Syntax supported](#syntax-supported)
- [Frequently Asked Questions](#frequently-asked-questions)

# Summary

bulk_examples_generator come in two flavors: binary or crate (library).

If you only want to generate examples see [Binary use](#binary-use)

If you want to use the crate for your own needs see [Crate use](#crate-use)

# Binary usage

Well! you just want to generate some examples or test this application.

## Requirements

- You have to install [Rust](https://www.rust-lang.org/) (Minimum version: 1.36.0)

**And it's all?**

Yes.

**Why there aren't executables available?**

As soon as this library becomes stable, I will create executables for Windows and Linux.

## Getting started

1. Install the application or clone the repository

```bash
cargo install bulk_examples_generator
```

If you clone the repository instead of using `bulk-examples-generator` you have to use `cargo run -- <comands>`

2. Create a PEST grammar and put on a file e.g. `mytest.pest`

```rust
// I like Rust!
language = {"Rust" | "Python" | "Go" | "Java" | "PHP" | "Haskell"}
one = {"1"}
daysNumber = {one ~ " day" | !one ~ ASCII_NONZERO_DIGIT ~ " days"}
sentence = {"I have been programming in " ~ language ~ " for " ~ daysNumber ~ "."}
```

3. Execute the following command:

```bash
bulk_examples_generator --grammar mytest.pest --quantity 3 --start-rule sentence --out-type stdout
```

or shorter

```bash
bulk_examples_generator -g mytest.pest -q 3 -s sentence -o stdout
```

You will have to see something like:

```
I have been programming in Rust for 3 days.
I have been programming in PHP for 5 days.
I have been programming in Haskell for 1 day.
```

## Usage

You need 4 parameters for use this application

1. A grammar in pest notation
1. A number (quantity of examples to generate)
1. A start rule (where begins the generation)
1. A output type (where save or print the examples)

```bash
bulk_examples_generator -g life.pest -q 3000 -s love -o stdout
```

### Grammars

To use Bulk examples generator you need to know how write grammars in PEST notation.

Take a look to [Quick guide for write grammars](WritingGrammars.md) or check the [pest reference](https://pest.rs/book/grammars/syntax.html)

**Note:** Not all pest syntax is supported, [check here](#syntax-supported)

You can verify the syntax of your grammar [here](https://pest.rs/#editor)

### Additional functionalities

The unique function added to generation that you can use in the grammar is the BLACKLIST generation

#### Blacklist generation

It's a deeper mechanism for avoid the generation of elements in certain rules

Suppose that you have this grammar:

```rust
Text = {"Random text"}
FlowContent = {Header | Main | Form | Section | Text }

// Header can't contains Main Element
Header = {"<header>" ~ FlowContent* ~ "</header>"}
Main = {"<main>" ~  FlowContent+ ~ "</main>"}
// Form can't contains Form Element
Form = {"<form>" ~ FlowContent* ~ "</form>"}
Section = {"<section>" ~ FlowContent+ ~ "</section>"}

```

This can generate problematic elements like this:

```HTML
<header>
    <main> <!-- main is forbidden at this point -->
        <section>
            <form>
            </form>
            Random text
        </section>
    </main>
</header>
```

To meet the restrictions you could do

```rust
Text = {"Random text"}
FlowContent = {Header | Main | Form | Section | Text }
FlowContentWithoutMain = {Header | Form | Section | Text }
FlowContentWithoutForm = {Header | Main | Section | Text }

Header = {"<header>" ~ FlowContentWithoutMain*  ~ "</header>"}
Main = {"<main>" ~  FlowContent+ ~ "</main>"}
Form = {"<form>" ~ FlowContentWithoutForm* ~ "</form>"}
Section = {"<section>" ~ FlowContent+ ~ "</section>"}
```

But this complicates the grammar if you have a lot of rules or restrictions.

What about of use the negation predicate like this?

```rust
Text = {"Random text"}
FlowContent = {Header | Main | Form | Section | Text }

Header = {"<header>" ~ (!Main ~ FlowContent)* ~ "</header>"}
Main = {"<main>" ~  FlowContent+ ~ "</main>"}
Form = {"<form>" ~ (!Form ~ FlowContent)* ~ "</form>"}
Section = {"<section>" ~ FlowContent+ ~ "</section>"}
```

The negation just works at first level, then other rules can generate main.

```HTML
<header>
    <section>
        <main> <!-- main still is forbidden at this point -->
            <section>
                <form>
                    Random text
                </form>
            </section>
        </main>
    </section>
</header>
```

The solution is use BLACKLIST generation:

```rust
Text = {"Random text"}
FlowContent = {Header | Main | Form | Section | Text }

Header = {"<header>" ~ "|BLACKLIST|I|Main|" ~ FlowContent* ~ "|BLACKLIST|R|Main|" ~ "</header>"}
Main = {"<main>" ~  FlowContent+ ~ "</main>"}
Form = {"<form>" ~ "|BLACKLIST|I|Form|" ~ FlowContent* ~ "|BLACKLIST|R|Form|" ~ "</form>"}
Section = {"<section>" ~ FlowContent+ ~ "</section>"}
```

```HTML
<!-- A correct generation -->
<main>
    <header>
        <section>
            <form></form>
        </section>
        Random textRandom text
    </header>
</main>
```

**How it works?**

Blacklist it's a list for avoid open rules in any level of the generation

You can add a rule like this `"|BLACKLIST|I|MyRule|"` or multiples rules at the same times like this `"|BLACKLIST|I|MyRule|OtherRule|"`

You can remove a rule like this `"|BLACKLIST|R|MyRule|"` or multiples rules at the same times like this `"|BLACKLIST|R|MyRule|OtherRule|"`

### Start rule

A start rule is required to begin the generation, if the start rule doesn't exist on the grammar, the examples will print the name of the rule

```
bulk_examples_generator -g mytest.pest -q 3 -s cookies -o stdout
```

```
cookies
cookies
cookies
```

### Output type (stdout, file, folder or debug)

#### Stdout

You can print the examples in stdout with the parameter `--out-type stdout` or `-o stdout`

```bash
bulk_examples_generator -g mytest.pest -q 3 -s sentence -o stdout
```

You will have to see something like:

```
I have been programming in Rust for 3 days.
I have been programming in PHP for 5 days.
I have been programming in Haskell for 1 day.
```

You can print the number of the example generated with `--print-progress` flag, the enumeration is not sequential because the generation is parallel

```bash
bulk_examples_generator -g mytest.pest -q 3 -s sentence -o stdout
```

You will have to see something like:

```
Example #2 generated:
I have been programming in Rust for 3 days.
Example #3 generated:
I have been programming in PHP for 5 days.
Example #1 generated:
I have been programming in Haskell for 1 day.
```

#### File

You can use ">" for save all examples in a file.

```bash
bulk_examples_generator -g mytest.pest -q 3 -s sentence -o stdout > "big-file.txt"
```

#### Folder

You can use `--out-type folder` or `-o folder` along with `--output-folder` to choose the folder and save the examples there (one file for each example).

```bash
bulk_examples_generator -g mytest.pest -q 3 -s sentence -o folder --output-folder examples
```

By default all files will have the name "example-{}.txt" where {} is the number of the example, you can change this with `--template-name` option

```bash
bulk_examples_generator -g mytest.pest -q 3 -s sentence -o folder --output-folder examples --template-name "book-number-{}.txt"
```

In this mode you will see a progress bar with the elapsed time and estimated remaining time.

#### Debug

Currently you can use `--out-type debug` or `-o debug` for print the options loaded, the grammar AST, the progress in the generation and finally print the generated examples in a vector.

```bash
bulk_examples_generator -g mytest.pest -q 3 -s sentence -o debug
```

Soon I expect to improve the debug mode with more information.

## Config file

Let's take the following grammar as an example:

```rust
// configGramar.pest
Body = {Element+}
Element = {Paragraph | Anchor | Text}
Paragraph = {"<p>" ~  Element* ~  "</p>"}
Anchor = {"<a>" ~ (Text | Paragraph)+ ~ "</a>"}
Text = {ASCII_ALPHA{1,5}}
```

Running a couple of times with

`bulk_examples_generator -g configGramar.pest -q 1 -o stdout -s Body`

You can see that the generation sometimes is very long, this is technically correct because the modifiers "\*" and "+" means zero or more and one or more, more means infinity but we do not have infinite time, so this is why there are upper limits.

To use the configuration just create a TOML file with the parameters and use the `--config-file` or `c` for load the file in the generation.

`bulk_examples_generator -g configGramar.pest -q 1 -o stdout -s Body -c config.toml`

There are 8 parameters that you can use in a config file

### Global parameters

**expand_limit**

Max rules opened in generation, when this limit is reached the generation of subsequent rules return the parameter text_expand_limit.

_default value:_ None (No limit)

**soft_limit**

To process a rule the elements are placed in a stack. If the grammar is very deep or recursive the number of elements in the stack will be large, if the length of the stack exceeds the value of this parameter then the delimiters "\*", "+" will be converted into ranges [0,1] and [1,2] respectively in order to reduce the number of items to process.

_default value:_ 10.000

**hard_limit**

In the process of generating an example, each processed expression increases the expression counter, if the parameter value is reached, all the unprocessed expressions from now on will not produce any results, the identifiers will only return the parameter text_expand_limit.

_default value:_ 25.000

**limit_depth_level**

All of the generation process of an example happens in a stack (There isn't recursion involved) except for a little expression `!b ~ a`.

If you have a recursive grammar with a lot of negations, the parameter limit_depth_level return the parameter text_expand_limit.

_default value:_ 200

### Expression parameters

| Parameter Description               | Description                                                                         | Default value |
| ----------------------------------- | ----------------------------------------------------------------------------------- | ------------- |
| text_expand_limit                   | It's the text returned by rules when the hard_limit or limit_depth_level is reached | ""            |
| upper_bound_zero_or_more_repetition | It's the upper limit in `rule*`                                                     | 5             |
| upper_bound_one_or_more_repetition  | It's the upper limit in `rule+`                                                     | 5             |
| upper_bound_at_least_repetition     | It's the upper limit in `rule{n,}`                                                  | 10            |
| max_attempts_negation               | Max attempts to generate `a` in `!b ~ a`                                            | 100           |

## Command line options

`bulk_examples_generator --help`

<details>
<summary>
Click to see the options
</summary>

```
USAGE:
    bulk-examples-generator.exe [FLAGS] [OPTIONS] --grammar <grammar> --out-type <out-type> --quantity <quantity> --start-rule <start-rule>

FLAGS:
    -h, --help
            Prints help information

        --print-progress
            Used when the out-type is stdout

            Print "Example #n generated:" before print the example

    -V, --version
            Prints version information


OPTIONS:
    -c, --config-file <config-file>
            Config file for generate elements, for more details pleaser refer to README Default config available in
            src/config/default.toml

    -g, --grammar <grammar>
            Path of grammar for generate examples

    -o, --out-type <out-type>
            Where to write the examples: one of debug, stdout, folder

            debug: Print results in stdout (vec form) for debugging purposes
            stdout: Print results in stdout
            folder: Create one file for each example (use template_name for personalize the filename)

        --output-folder <output-folder>
            Output folder to save the examples

    -q, --quantity <quantity>
            Quantity of examples to generate

    -s, --start-rule <start-rule>
            Rule to start generation of examples

    -t, --template-name <template-name>
            Name of the files, e.g. html-test-{}.html, {} will be used for enumerating the example [default:
            example-{}.txt]
```

</details>

## Benchmarks

**How fast is bulk examples generator?**

Currently the application generates examples using parallelism at example level, It means that if you have four logical cores, probably your pc can generate 4 examples at the same time giving a speed boost in the generation.

You can execute a basic benchmark with the following command (the getting started example) for test the speed in the generation of 100 examples.

**Note:** remember that for get an accurate result of your benchmark you have to close all unnecessary programs and have your laptop connected to electricity

`cargo bench -- --verbose --measurement-time 120`

In my laptop the results are [min mean max]:

- Intel i5 6198DU (2 Physic cores/4 logical cores)
- DDR4 16GB
- SSD Sata Western Digital

```
Benchmarking Readme grammar example: Warming up for 3.0000 s
Benchmarking Readme grammar example: Collecting 100 samples in estimated 138.32 s (20200 iterations)
Benchmarking Readme grammar example: Analyzing
Readme grammar example  time:   [7.9290 ms 8.0080 ms 8.0937 ms]

slope  [7.9290 ms 8.0937 ms] R^2            [0.6422301 0.6410218]
mean   [7.5284 ms 7.8428 ms] std. dev.      [747.06 us 860.73 us]
median [7.6470 ms 8.0680 ms] med. abs. dev. [686.19 us 1.3668 ms]
```

This test doesn't include the time required for print or save the examples, just the time that take generate the examples.

If I extrapolate the results (Don't do this, is just an example), we have the following times:

| Examples   | Estimated time  |
| ---------- | --------------- |
| 100        | 8 milliseconds  |
| 1.000      | 80 milliseconds |
| 10.000     | 0,8 seconds     |
| 100.000    | 8 seconds       |
| 1.000.000  | 80 seconds      |
| 10.000.000 | 13,3 minutes    |

It's an acceptable time for a basic grammar, I will add more benchmarks with bigger grammars later

**It could be faster?**

Yes, currently the parallelism is at example level, but there are rules that could be generated in a parallel way increasing the speed of generation on bigger grammars. At the moment I want to correct the errors that the application may present.

# Crate usage

## Getting started

It's just add in Cargo.toml

```TOML
bulk_examples_generator = "^0.1"
```

### Example

```rust

// Default configuration for the generator
let mut config: GeneratorConfig = Default::default();

// Grammar string
let mut grammar = r#"
        language = {"Rust" | "Python" | "Go" | "Java" | "PHP" | "Haskell"}
        one = {"1"}
        daysNumber = {one ~ " day" | !one ~ ASCII_NONZERO_DIGIT ~ " days"}
        sentence = {"I have been programming in " ~ language ~ " for " ~ daysNumber ~ "."}
    "#;

// Generate the examples
let results = parallel_generate_examples(
            grammar.to_string(),        // The grammar
            5,                          // Quantity of examples
            "sentences".to_string(),    // Start rule
            &config,                    // Config of the generator
            false,                      // Print progress
            false,                      // Print in stdout, false return a vector with the examples
        );

println!("{:?}", results);

```

### Config

```rust
// Default configuration for the generator
let mut config: GeneratorConfig = Default::default();

// Change the configuration
config.upper_bound_one_or_more_repetition = 20;

// Or load a config from TOML file
let mut config_file = GeneratorConfig::new("config.toml").unwrap();
```

### Available functions

Currently there are 4 functions available

`parallel_generate_examples`

The function used for generate examples. [docs](https://docs.rs/bulk-examples-generator/0.1.0/serde/fn.parallel_generate_examples.html)

`parallel_generate_save_examples`

The function used for generate and save the examples. [docs](https://docs.rs/bulk-examples-generator/0.1.0/fn.parallel_generate_save_examples.html)

`compile_grammar`

Compile a grammar string and creates a HashMap with rules founds as keys and their components as entries. It can be useful for see the AST or check the validity of the example generated [docs](https://docs.rs/bulk-examples-generator/0.1.0/fn.compile_grammar.html)

`parse_input`

Parse an example generated with the grammar provided, is a symlink to parse function of pest. Useful for check the validity of an example [docs](https://docs.rs/bulk-examples-generator/0.1.0/fn.parse_input.html)

## Syntax supported

Adapted from pest reference.

The ✔️ mark means that at moment the syntax is supported and I tested the syntax at basic/medium level (If you find bugs, please create an issue!).

The ❓ mark means that at moment the support is unknown, maybe I coded the support but I didn't test with any grammar.

The ❌ mark means that at the moment isn't supported. I know that I didn't code any related to this syntax.

|         Syntax          |              Meaning              | Supported | Observations                                                            |
| :---------------------: | :-------------------------------: | --------- | ----------------------------------------------------------------------- |
|     `foo = { ... }`     |          [regular rule]           | ✔️        |                                                                         |
|    `bar = _{ ... }`     |             [silent]              | ❓        |                                                                         |
|    `baz = @{ ... }`     |             [atomic]              | ❓        |                                                                         |
|    `qux = ${ ... }`     |         [compound-atomic]         | ❓        |                                                                         |
|   `plugh = !{ ... }`    |           [non-atomic]            | ❓        |                                                                         |
|  Built-in ascii rules   |      [Built-in ascii rules]       | ✔️        |                                                                         |
| Built-in unicode rules  |     [Built-in unicode rules]      | ❌        |                                                                         |
|         `"abc"`         |          [exact string]           | ✔️        |                                                                         |
|        `^"abc"`         |        [case insensitive]         | ❓        | Supported but not tested yet                                            |
|       `'a'..'z'`        |         [character range]         | ✔️        |                                                                         |
|          `ANY`          |          [any character]          | ❓        | Supported but not tested completely                                     |
|       `foo ~ bar`       |            [sequence]             | ✔️        |                                                                         |
| <code>baz \| qux</code> |         [ordered choice]          | ✔️        |                                                                         |
|         `foo*`          |          [zero or more]           | ✔️        | There is a parameter for change the upper limit                         |
|         `bar+`          |           [one or more]           | ✔️        | There is a parameter for change the upper limit                         |
|         `baz?`          |            [optional]             | ✔️        |                                                                         |
|        `qux{n}`         |           [exactly *n*]           | ✔️        |                                                                         |
|       `qux{m, n}`       | [between *m* and *n* (inclusive)] | ✔️        |                                                                         |
|        `qux{m,}`        |          [At least *m* ]          | ✔️        | There is a parameter for change the upper limit                         |
|        `qux{,n}`        |     [At most *n* (inclusive)]     | ✔️        |                                                                         |
|         `&foo`          |       [positive predicate]        | ❌        |                                                                         |
|         `!bar`          |       [negative predicate]        | ✔️ ❗     | It's supported with a brute force approach in expressions like `!b ~ a` |
|       `PUSH(baz)`       |         [match and push]          | ❌        |                                                                         |
|          `POP`          |          [match and pop]          | ❌        |                                                                         |
|         `PEEK`          |        [match without pop]        | ❌        |                                                                         |

# Frequently Asked Questions

**Why there aren't executables available?**

As soon as this library becomes stable, I will create executables for Windows and Linux.

**How fast is bulk examples generator?**

Check the basic benchmark [here](#benchmarks)

**How it works? the code is recursive?**

Essentially the code constructs the parse tree of the grammar and traverse the tree to generate random elements according to the expressions; No, the most of the code is not recursive, all the generation process of an example happens in a stack, except for negation expressions `!b ~ a`.

**Can I use pest for parsing the examples generated?**

Yes and no, the most grammars can be parsable but there are examples that cannot be.

The most simple grammar that cannot be parsable in all examples generated is this:

```rust
tricky = {("a" | "ab") ~ "c"}
// Generate
// ac -> parseable
// abc -> not parseable
```

This happens because the operator "|" means ordered choice then the parser tries to parse first "a" then "bc" fails.

To resolve this issue, follow the pest recommendations of when writing a grammar

> In general, when writing a parser with choices, put the longest or most specific choice first, and the shortest or most general choice last.

https://pest.rs/book/grammars/peg.html#ordered-choice

Applying the advice the new grammar would look like this:

```rust
easy = {("ab" | "a") ~ "c"}
// Generate
// ac -> parseable
// abc -> parseable
```

Other example is [non-backtracking](https://pest.rs/book/grammars/peg.html#non-backtracking)

Issues related:

- https://github.com/pest-parser/pest/issues/209
- https://github.com/pest-parser/pest/issues/308

**I want to generate a million of examples using the crate, why not do you return a stream or observer/listener model or something like that?**

Sorry for that. Currently I want to improve/test the syntax grammar and simplify the code, if you want to generate this amount of examples please call the functions with batches and create an issue for know the interest about that.
