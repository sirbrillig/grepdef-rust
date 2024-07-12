use clap::Parser;
use regex::Regex;
use std::error::Error;
use std::fs;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
pub struct Args {
    /// The symbol name to search for
    pub query: String,

    /// The file path(s) to search
    pub file_path: String,

    /// The file type to search (js, ts, php)
    #[arg(short = 't', long = "type")]
    pub file_type: String,

    /// Show line numbers (starting with 1).
    #[arg(short = 'n', long = "line-number")]
    pub line_number: bool,
}

pub struct Config {
    query: String,
    file_path: String,
    file_type: FileType,
    line_number: bool,
}

impl Config {
    pub fn new(args: Args) -> Result<Config, &'static str> {
        Ok(Config {
            query: args.query,
            file_path: args.file_path,
            file_type: FileType::from_string(args.file_type)?,
            line_number: args.line_number,
        })
    }
}

pub enum FileType {
    JS,
}

impl FileType {
    pub fn from_string(file_type_string: String) -> Result<FileType, &'static str> {
        match file_type_string.as_str() {
            "js" => Ok(FileType::JS),
            _ => {
                return Err("Invalid file type");
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub text: String,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    for line in search(&config)? {
        if config.line_number {
            println!("{}:{}:{}", line.file_path, line.line_number, line.text);
        } else {
            println!("{}:{}", line.file_path, line.text);
        }
    }
    Ok(())
}

fn get_regexp_for_file_type(file_type: &FileType) -> Regex {
    let regexp_string = match file_type {
        FileType::JS => &format!(r"\.(js|jsx|ts|tsx|mjs|cjs)$"),
    };
    Regex::new(regexp_string).unwrap()
}

fn get_regexp_for_query(query: &str, file_type: &FileType) -> Regex {
    let regexp_string = match file_type {
        FileType::JS => &format!(
            r"(\b(function|var|let|const|class|interface|type)\s+{query}\b|\b{query}\([^)]*\)\s*(:[^\{{]+)?\{{|\b{query}:|@typedef\s*(\{{[^\}}]+\}})?\s*{query}\b)"
        ),
    };
    Regex::new(regexp_string).unwrap()
}

fn does_file_path_match_type(path: &str, file_type: &FileType) -> bool {
    let re = get_regexp_for_file_type(file_type);
    re.is_match(path)
}

pub fn search(config: &Config) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let mut results = vec![];

    for entry in WalkDir::new(&config.file_path) {
        let path = entry?.into_path();
        if path.is_dir() {
            continue;
        }
        let path = match path.to_str() {
            Some(p) => p.to_string(),
            None => return Err("Error getting string from path".into()),
        };
        if !does_file_path_match_type(&path, &config.file_type) {
            continue;
        }
        let search_result = search_file(&config.query, &config.file_type, &path);
        match search_result {
            Ok(result_entries) => results.extend(result_entries),
            Err(err) => return Err(err),
        }
    }

    return Ok(results);
}

fn search_file(
    query: &str,
    file_type: &FileType,
    file_path: &str,
) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let contents = fs::read_to_string(&file_path)?;
    let re = get_regexp_for_query(query, file_type);

    Ok(contents
        .lines()
        .enumerate()
        .map(|(index, line)| SearchResult {
            file_path: String::from(file_path),
            line_number: index + 1,
            text: String::from(line),
        })
        .filter(|result| re.is_match(&result.text))
        .collect())
}
