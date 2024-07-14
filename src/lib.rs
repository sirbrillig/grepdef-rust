use clap::Parser;
use colored::Colorize;
use ignore::Walk;
use memchr::memmem;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::io::{self, BufRead, Read, Seek};
use strum_macros::EnumString;

#[derive(Parser, Debug)]
pub struct Args {
    /// The symbol name to search for
    pub query: String,

    /// The file path(s) to search
    pub file_path: Option<String>,

    /// The file type to search (js, ts, php)
    #[arg(short = 't', long = "type")]
    pub file_type: String,

    /// Show line numbers (starting with 1).
    #[arg(short = 'n', long = "line-number")]
    pub line_number: bool,

    /// (Advanced) The searching method
    #[arg(long = "search-method")]
    pub search_method: Option<SearchMethod>,
}

#[derive(clap::ValueEnum, Clone, Default, Debug, EnumString, PartialEq)]
pub enum SearchMethod {
    #[default]
    PrescanRegex,
    PrescanMemmem,
    NoPrescan,
}

pub struct Config {
    query: String,
    file_path: String,
    file_type: FileType,
    line_number: bool,
    search_method: SearchMethod,
}

impl Config {
    pub fn new(args: Args) -> Result<Config, &'static str> {
        Ok(Config {
            query: args.query,
            file_path: args.file_path.unwrap_or(".".into()),
            file_type: FileType::from_string(args.file_type)?,
            line_number: args.line_number,
            search_method: args.search_method.unwrap_or_default(),
        })
    }
}

pub enum FileType {
    JS,
    PHP,
}

impl FileType {
    pub fn from_string(file_type_string: String) -> Result<FileType, &'static str> {
        match file_type_string.as_str() {
            "js" => Ok(FileType::JS),
            "php" => Ok(FileType::PHP),
            _ => Err("Invalid file type"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub text: String,
}

fn get_regexp_for_file_type(file_type: &FileType) -> Regex {
    let regexp_string = match file_type {
        FileType::JS => &r"\.(js|jsx|ts|tsx|mjs|cjs)$".to_string(),
        FileType::PHP => &r"\.php$".to_string(),
    };
    Regex::new(regexp_string).unwrap()
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    for line in search(&config)? {
        if config.line_number {
            println!(
                "{}:{}:{}",
                line.file_path.magenta(),
                line.line_number.to_string().green(),
                line.text
            );
        } else {
            println!("{}:{}", line.file_path.magenta(), line.text);
        }
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
    Regex::new(regexp_string).unwrap()
}

pub fn search(config: &Config) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let re = get_regexp_for_query(&config.query, &config.file_type);
    let file_type_re = get_regexp_for_file_type(&config.file_type);
    let mut results = vec![];

    for entry in Walk::new(&config.file_path) {
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
        let search_result = search_file(&re, &path, config);
        match search_result {
            Ok(result_entries) => results.extend(result_entries),
            Err(err) => return Err(err),
        }
    }

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
            full = split_full.next().unwrap().to_vec();
        }
        full.extend(buf);
        if finder.find(&full).is_some() {
            break true;
        }
    }
}

fn search_file(
    re: &Regex,
    file_path: &str,
    config: &Config,
) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let mut file = fs::File::open(file_path)?;

    // Scan the file in big chunks to see if it has what we are looking for. This is more efficient
    // than going line-by-line on every file since matches should be quite rare.
    if match config.search_method {
        SearchMethod::PrescanRegex => !does_file_match_regexp(&file, re),
        SearchMethod::PrescanMemmem => !does_file_match_query(&file, &config.query),
        SearchMethod::NoPrescan => false,
    } {
        return Ok(vec![]);
    }

    file.rewind()?;
    Ok(search_file_line_by_line(re, file_path, &file))
}

fn search_file_line_by_line(re: &Regex, file_path: &str, file: &fs::File) -> Vec<SearchResult> {
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
                line_number: line_counter,
                text: text.trim().into(),
            })
        })
        .collect()
}
