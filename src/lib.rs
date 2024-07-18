#![warn(missing_docs)]
//! Quick search for symbol definitions in various programming languages
//!
//! Currently this supports JS (or TypeScript) and PHP.
//!
//! This can be used like "Go to definition" in an IDE, except that instead of using a language
//! server, it just searches for the definition using text parsing. This is less accurate but often
//! faster in projects with lots of files or where a language server won't work or hasn't yet
//! started.
//!
//! GrepDef since v2 is written in Rust and is designed to be extremely fast.
//!
//! This can also be used as a library crate for other Rust programs.
//!
//! # Example
//!
//! The syntax of the CLI is similar to that of `grep` or `ripgrep`: first put the symbol you want
//! to search for (eg: a function name, class name, etc.) and then list the file(s) or directories
//! over which you want to search.
//!
//! ```text
//! $ grepdef parseQuery ./src
//! // ./src/queries.js:function parseQuery {
//! ```
//!
//! Just like `grep`, you can add the `-n` option to include line numbers.
//!
//! ```text
//! $ grepdef -n parseQuery ./src
//! // ./src/queries.js:17:function parseQuery {
//! ```
//!
//! The search will be faster if you specify what type of file you are searching for using the
//! `--type` option.
//!
//! ```text
//! $ grepdef --type js -n parseQuery ./src
//! // ./src/queries.js:17:function parseQuery {
//! ```
//!
//! To use the crate from other Rust code, use the [search] function.
//!
//! ```
//! use grepdef_rust::{search, Args, Config};
//!
//! let config = Config::new(Args {
//!     query: String::from("parseQuery"),
//!     ..Args::default()
//! })
//! .unwrap();
//! for result in search(&config).unwrap() {
//!     println!("{}", result.to_grep());
//! }
//! ```

use clap::Parser;
use colored::Colorize;
use ignore::Walk;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead, Seek};
use std::sync::Arc;
use std::sync::Mutex;
use strum_macros::Display;
use strum_macros::EnumString;

mod file_type;
mod threads;

/// The command-line arguments to be turned into a [Config]
///
/// Use this instead of [Config] directly if you want to benefit from optional parameters and
/// auto-detection.
///
/// Can be passed to [Config::new()].
///
/// The only required property is [Args::query].
///
/// # Example
///
/// ```
/// use grepdef_rust::{Config, Args};
/// let config = Config::new(Args {
///     query: String::from("parseQuery"),
///     line_number: true,
///     ..Args::default()
/// });
/// ```
#[derive(Parser, Debug, Default)]
pub struct Args {
    /// (Required) The symbol name (function, class, etc.) to search for
    pub query: String,

    /// The file path(s) to search; recursively searches directories and respects .gitignore
    pub file_path: Option<Vec<String>>,

    /// The file type to search (js, php); will guess if not set but this is slower
    #[arg(short = 't', long = "type")]
    pub file_type: Option<String>,

    /// Show line numbers of matches if set
    #[arg(short = 'n', long = "line-number")]
    pub line_number: bool,

    /// Disable color (also supports NO_COLOR env)
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// (Advanced) Print debugging information
    #[arg(long = "debug")]
    pub debug: bool,

    /// (Advanced) The searching method
    #[arg(long = "search-method")]
    pub search_method: Option<SearchMethod>,
}

/// (Advanced) The type of underlying search algorithm to use
///
/// In general, a pre-scan is a good idea to quickly skip files that don't have a match, which
/// should be most files. You shouldn't need to change this from the default.
#[derive(clap::ValueEnum, Clone, Default, Debug, EnumString, PartialEq, Display)]
pub enum SearchMethod {
    /// Pre-scan each file by reading fully into memory and using a Regex
    #[default]
    PrescanRegex,

    /// Pre-scan each file by reading bytes until the query is found using memmem
    PrescanMemmem,

    /// Don't pre-scan files.
    NoPrescan,
}

/// The configuration passed to the [search] function
///
/// The easiest way to use this is to first create an [Args] object and pass that to
/// [Config::new] to take advantage of optional properties and auto-detection.
///
/// # Example
///
/// ```
/// use grepdef_rust::{Config, Args};
/// let config = Config::new(Args { query: String::from("parseQuery"), ..Args::default() });
/// ```
#[derive(Clone, Debug)]
pub struct Config {
    /// The symbol name (function, class, etc.) being searched for
    pub query: String,

    /// The list of file paths to search, ignoring invisible or gitignored files
    pub file_paths: Vec<String>,

    /// The type of files to scan (JS or PHP)
    pub file_type: FileType,

    /// Include line numbers in results if true
    pub line_number: bool,

    /// Output debugging info during search if true
    pub debug: bool,

    /// Explicitly disable color output if true
    pub no_color: bool,

    /// The [SearchMethod] to use
    pub search_method: SearchMethod,
}

impl Config {
    /// Create a new Config using an [Args]
    pub fn new(args: Args) -> Result<Config, &'static str> {
        if args.debug {
            let args_formatted = format!("Creating config with args {:?}", args);
            println!("{}", args_formatted.yellow());
        }
        let file_paths = match args.file_path {
            Some(file_path) => file_path,
            None => vec![".".into()],
        };
        let file_type = match args.file_type {
            Some(file_type_string) => FileType::from_string(file_type_string)?,
            None => file_type::guess_file_type(&file_paths)?,
        };
        let config = Config {
            query: args.query,
            file_paths,
            file_type,
            line_number: args.line_number,
            debug: args.debug,
            no_color: args.no_color,
            search_method: args.search_method.unwrap_or_default(),
        };
        debug(&config, format!("Created config {:?}", config).as_str());
        Ok(config)
    }
}

/// The supported file types to search
///
/// You can turn a string into a [FileType] using [FileType::from_string] which also supports
/// type aliases like `javascript`, `javascriptreact`, or `typescript.tsx`.
#[derive(Clone, Debug)]
pub enum FileType {
    /// The JS (or TS) file type
    JS,

    /// The PHP file type
    PHP,
}

impl FileType {
    /// Turn a string into a [FileType]
    ///
    /// You can turn a string into a [FileType] using [FileType::from_string] which also supports
    /// type aliases like `javascript`, `javascriptreact`, or `typescript.tsx`.
    pub fn from_string(file_type_string: String) -> Result<FileType, &'static str> {
        match file_type_string.as_str() {
            "js" => Ok(FileType::JS),
            "ts" => Ok(FileType::JS),
            "jsx" => Ok(FileType::JS),
            "tsx" => Ok(FileType::JS),
            "javascript" => Ok(FileType::JS),
            "javascript.jsx" => Ok(FileType::JS),
            "javascriptreact" => Ok(FileType::JS),
            "typescript" => Ok(FileType::JS),
            "typescript.tsx" => Ok(FileType::JS),
            "typescriptreact" => Ok(FileType::JS),
            "php" => Ok(FileType::PHP),
            _ => Err("Invalid file type"),
        }
    }
}

/// A result from calling [search]
///
/// The `line_number` will be set only if [Config::line_number] is true when calling [search].
///
/// See [SearchResult::to_grep] as the most common formatting output.
#[derive(Debug, PartialEq, Clone)]
pub struct SearchResult {
    /// The path to the file containing the symbol definition
    pub file_path: String,

    /// The line number of the symbol definition in the file
    pub line_number: Option<usize>,

    /// The symbol definition line
    pub text: String,
}

impl SearchResult {
    /// Return a formatted string for output in the "grep" format
    ///
    /// That is, either `file path:text on line` or, if [Config::line_number] is true,
    /// `file path:line number:text on line`.
    ///
    /// # Example
    ///
    /// If [Config::line_number] is true,
    ///
    /// ```text
    /// ./src/queries.js:17:function parseQuery {
    /// ```
    pub fn to_grep(&self) -> String {
        match self.line_number {
            Some(line_number) => format!(
                "{}:{}:{}",
                self.file_path.magenta(),
                line_number.to_string().green(),
                self.text
            ),
            None => format!("{}:{}", self.file_path.magenta(), self.text),
        }
    }
}

fn get_regexp_for_query(query: &str, file_type: &FileType) -> Regex {
    let regexp_string = match file_type {
        FileType::JS => &format!(
            r"(\b(function|var|let|const|class|interface|type)\s+{query}\b|\b{query}\([^)]*\)\s*(:[^\{{]+)?\{{|\b{query}:|@typedef\s*(\{{[^\}}]+\}})?\s*{query}\b)"
        ),
        FileType::PHP => &format!(r"\b(function|class|trait|interface|enum) {query}\b"),
    };
    Regex::new(regexp_string).expect("Could not create regex for file type query")
}

/// Search for a symbol definition
///
/// This is the main API of this crate.
///
/// # Example
///
/// ```
/// use grepdef_rust::{search, Args, Config};
/// let config = Config::new(Args {
//     query: String::from("parseQuery"),
///     line_number: true,
///     ..Args::default()
/// })
/// .unwrap();
/// for result in search(&config).unwrap() {
///     println!("{}", result.to_grep());
/// }
/// ```
pub fn search(config: &Config) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let re = get_regexp_for_query(&config.query, &config.file_type);
    let file_type_re = file_type::get_regexp_for_file_type(&config.file_type);
    let mut pool = threads::ThreadPool::new(5);
    let results: Vec<SearchResult> = vec![];
    let results = Arc::new(Mutex::new(results));

    debug(config, "Starting searchers");
    for file_path in &config.file_paths {
        for entry in Walk::new(file_path) {
            let path = entry?.into_path();
            if path.is_dir() {
                continue;
            }
            let path = match path.to_str() {
                Some(p) => p.to_string(),
                None => return Err("Error getting string from path".into()),
            };
            if !file_type_re.is_match(&path) {
                continue;
            }

            let re1 = re.clone();
            let path1 = path.clone();
            let config1 = config.clone();
            let results1 = Arc::clone(&results);
            pool.execute(move || {
                search_file(
                    &re1,
                    &path1,
                    &config1,
                    move |file_results: Vec<SearchResult>| {
                        results1
                            .lock()
                            .expect("Unable to collect search data from thread")
                            .extend(file_results);
                    },
                );
            })
        }
    }

    debug(config, "Waiting for searchers to complete");
    pool.wait_for_all_jobs_and_stop();
    debug(config, "Searchers complete");

    let results = Arc::into_inner(results)
        .expect("Unable to collect search results from threads: reference counter failed");
    let results = results
        .into_inner()
        .expect("Unable to collect search results from threads: mutex failed");
    Ok(results)
}

fn debug(config: &Config, output: &str) {
    if config.debug {
        println!("{}", output.yellow());
    }
}

fn search_file<F>(re: &Regex, file_path: &str, config: &Config, callback: F)
where
    F: FnOnce(Vec<SearchResult>) + Send + 'static,
{
    debug(config, format!("Scanning file {}", file_path).as_str());
    let file = fs::File::open(file_path);

    match file {
        Ok(mut file) => {
            // Scan the file in big chunks to see if it has what we are looking for. This is more efficient
            // than going line-by-line on every file since matches should be quite rare.
            debug(
                config,
                format!("  Using search-method {}", config.search_method).as_str(),
            );
            if match config.search_method {
                SearchMethod::PrescanRegex => !file_type::does_file_match_regexp(&file, re),
                SearchMethod::PrescanMemmem => {
                    !file_type::does_file_match_query(&file, &config.query)
                }
                SearchMethod::NoPrescan => false,
            } {
                debug(config, "  Presearch found no match; skipping");
                callback(vec![]);
                return;
            }

            let rewind_result = file.rewind();
            if rewind_result.is_err() {
                callback(vec![]);
                return;
            }
            debug(config, "  Presearch was successful; searching for line");
            callback(search_file_line_by_line(re, file_path, &file, config));
        }
        Err(_) => {
            callback(vec![]);
        }
    }
}

fn search_file_line_by_line(
    re: &Regex,
    file_path: &str,
    file: &fs::File,
    config: &Config,
) -> Vec<SearchResult> {
    let lines = io::BufReader::new(file).lines();
    let mut line_counter = 0;

    lines
        .filter_map(|line| {
            line_counter += 1;
            if !match &line {
                Ok(line) => re.is_match(line),
                Err(_) => false,
            } {
                return None;
            }

            let text = match line {
                Ok(line) => line,
                // If reading the line causes an error (eg: invalid UTF), then skip it by treating
                // it as empty.
                Err(_err) => String::from(""),
            };

            Some(SearchResult {
                file_path: String::from(file_path),
                line_number: if config.line_number {
                    Some(line_counter)
                } else {
                    None
                },
                text: text.trim().into(),
            })
        })
        .collect()
}
