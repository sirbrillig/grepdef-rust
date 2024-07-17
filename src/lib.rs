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
//! To use the crate from other Rust code, use the `search()` function.
//!
//! ```
//! use grepdef_rust::{search, Args, Config};
//! let config = Config::new(Args {
//!     query: String::from("parseQuery"),
//!     ..Args::default()
//!     })
//!     .unwrap();
//! for result in search(&config).unwrap() {
//!     println!("{}", result.to_grep());
//! }
//! ```

use clap::Parser;
use colored::Colorize;
use ignore::Walk;
use memchr::memmem;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead, Read, Seek};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use strum_macros::Display;
use strum_macros::EnumString;

/// The command-line arguments to be turned into a `Config`
///
/// Use this instead of `Config` directly if you want to benefit from optional parameters and
/// auto-detection.
///
/// Can be passed to `Config::new()`.
///
/// The only required property is `query`.
///
/// # Example
///
/// ```
/// use grepdef_rust::{Config, Args};
/// let config = Config::new(Args { query: String::from("parseQuery"), line_number: true, ..Args::default() });
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
#[derive(clap::ValueEnum, Clone, Default, Debug, EnumString, PartialEq, Display)]
pub enum SearchMethod {
    #[default]
    PrescanRegex,
    PrescanMemmem,
    NoPrescan,
}

/// The configuration passed to the `search()` function
///
/// The easiest way to use this is to first create an `Args` object and pass that to
/// `Config::new()` to take advantage of optional properties and auto-detection.
///
/// # Example
///
/// ```
/// use grepdef_rust::{Config, Args};
/// let config = Config::new(Args { query: String::from("parseQuery"), line_number: true, ..Args::default() });
/// ```
#[derive(Clone, Debug)]
pub struct Config {
    pub query: String,
    pub file_paths: Vec<String>,
    pub file_type: FileType,
    pub line_number: bool,
    pub debug: bool,
    pub no_color: bool,
    pub search_method: SearchMethod,
}

impl Config {
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
            None => guess_file_type(&file_paths)?,
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
/// You can turn a string into a `FileType` using `FileType::from_string()` which also supports
/// type aliases like `javascript`, `javascriptreact`, or `typescript.tsx`.
#[derive(Clone, Debug)]
pub enum FileType {
    JS,
    PHP,
}

impl FileType {
    /// Turn a string into a `FileType`
    ///
    /// You can turn a string into a `FileType` using `FileType::from_string()` which also supports
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

/// A result from calling `search()`
///
/// The `line_number` will be set only if `Config.line_number` is true when calling `search()`.
///
/// See `to_grep()` as the most common formatting output.
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
    /// That is, either `file path:text on line` or, if `Config.line_number` is true,
    /// `file path:line number:text on line`.
    ///
    /// # Example
    ///
    /// If `Config.line_number` is true,
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

fn guess_file_type(file_paths: &Vec<String>) -> Result<FileType, &'static str> {
    for file_path in file_paths {
        let guess = guess_file_type_from_file_path(file_path);
        if let Some(value) = guess {
            return Ok(value);
        }
    }
    Err("Unable to guess file type. Try using --type.")
}

fn guess_file_type_from_file_path(file_path: &str) -> Option<FileType> {
    let js_regex = get_regexp_for_file_type(&FileType::JS);
    let php_regex = get_regexp_for_file_type(&FileType::PHP);
    for entry in Walk::new(file_path) {
        let path = match entry {
            Ok(path) => path.into_path(),
            Err(_) => continue,
        };
        if path.is_dir() {
            continue;
        }
        let path = match path.to_str() {
            Some(p) => p.to_string(),
            None => String::from(""),
        };
        if js_regex.is_match(&path) {
            return Some(FileType::JS);
        }
        if php_regex.is_match(&path) {
            return Some(FileType::PHP);
        }
    }
    None
}

fn get_regexp_for_file_type(file_type: &FileType) -> Regex {
    let regexp_string = match file_type {
        FileType::JS => &r"\.(js|jsx|ts|tsx|mjs|cjs)$".to_string(),
        FileType::PHP => &r"\.php$".to_string(),
    };
    Regex::new(regexp_string).expect("Could not create regex for file extension")
}

/// Run the CLI script
///
/// This should not be used manually by other crates. See `search()` instead.
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if config.no_color {
        colored::control::set_override(false);
    }
    for line in search(&config)? {
        println!("{}", line.to_grep());
    }
    Ok(())
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

type WorkerReceiver = Arc<Mutex<mpsc::Receiver<Job>>>;
type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: WorkerReceiver) -> Worker {
        let thread = thread::spawn(move || loop {
            // recv will block until the next job is sent.
            let message = receiver
                .lock()
                .expect("Worker thread could not get message from main thread")
                .recv();

            match message {
                Ok(job) => {
                    job();
                }
                // The thread will stop when the job channel is sent an Err, which will happen when
                // the channel is closed.
                Err(_) => {
                    break;
                }
            }
        });
        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The count is the number of threads availalable in the pool.
    ///
    /// # Panics
    ///
    /// This function will panic if the size is 0.
    pub fn new(count: usize) -> ThreadPool {
        assert!(count > 0);

        // This channel is used to send Jobs to each thread.
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(count);

        for id in 0..count {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender
            .as_ref()
            .expect("Executing search thread failed")
            .send(job)
            .expect("Unable to send data to search thread");
    }

    pub fn wait_for_all_jobs_and_stop(&mut self) {
        // Close the Jobs channel which will trigger each thread to stop when it finishes its
        // current work.
        drop(self.sender.take());

        for worker in &mut self.workers {
            // Collect each thread which all should have stopped working by now.
            if let Some(thread) = worker.thread.take() {
                thread.join().expect("Unable to close thread");
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.wait_for_all_jobs_and_stop();
    }
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
    let file_type_re = get_regexp_for_file_type(&config.file_type);
    let mut pool = ThreadPool::new(4);
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

fn does_file_match_regexp(mut file: &fs::File, re: &Regex) -> bool {
    let mut buf = String::new();
    let bytes = file.read_to_string(&mut buf);
    if bytes.unwrap_or(0) == 0 {
        return false;
    }
    re.is_match(&buf)
}

fn does_file_match_query(mut file: &fs::File, query: &str) -> bool {
    let mut full: Vec<u8> = vec![];
    let mut buf = [0u8; 2048];
    let finder = memmem::Finder::new(query);
    loop {
        let bytes = file.read(&mut buf);
        if bytes.unwrap_or(0) == 0 {
            break false;
        }
        if full.contains(&0xA) {
            let mut split_full = full.rsplit(|&b| b == b'\n');
            full = split_full.next().unwrap_or(&[0u8, 1]).to_vec();
        }
        full.extend(buf);
        if finder.find(&full).is_some() {
            break true;
        }
    }
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
                SearchMethod::PrescanRegex => !does_file_match_regexp(&file, re),
                SearchMethod::PrescanMemmem => !does_file_match_query(&file, &config.query),
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
