use config::ConfigError;
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

enum OutputType {
    Debug,
    Stdout,
    File,
    Folder,
}

/// Struct for define the config of the execution
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExecutorConfig {
    /// This parameter is just for avoid print elements or create a vec with them, because we only want to benchmark the generation
    pub benchmark_mode: bool,
    /// Enable or disable parallel creation, default: true
    pub parallel_mode: bool,
    /// Print progress bar
    pub print_progress_bar: bool,
    /// Print progress text
    pub print_progress_text: bool,
    /// Print additional info
    pub print_debug: bool,
    /// Print every example in stdout
    pub print_stdout: bool,
    /// Create one file for all examples
    pub print_file: Option<PathBuf>,
    /// Create a file for every example
    /// (name_format, path)
    /// Name of the files, e.g. html-test-{}.html, {} will be used for enumerating the example
    pub print_folder: Option<(String, PathBuf)>,
    /// Return all examples generated in a vec
    pub return_vec: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        ExecutorConfig {
            benchmark_mode: false,
            parallel_mode: true,
            print_progress_bar: false,
            print_progress_text: false,
            print_debug: false,
            print_stdout: true,
            print_file: None,
            print_folder: None,
            return_vec: false,
        }
    }
}

impl ExecutorConfig {
    /// Create a config for benchmark, It's just change the parameter `benchmark_mode`
    ///
    /// `ExecutorConfig::benchmark()`
    ///
    /// If you want to get default config
    ///
    /// `let default: ExecutorConfig = Default::default();`
    ///
    pub fn benchmark() -> Self {
        let mut settings: Self = Default::default();
        settings.benchmark_mode = true;
        settings
    }
}
