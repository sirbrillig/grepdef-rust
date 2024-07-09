use clap::Parser;
use regex::Regex;
use std::error::Error;
use std::fs;

#[derive(Parser, Debug)]
pub struct Args {
    /// The symbol name to search for
    pub query: String,

    /// The file path(s) to search
    pub file_path: String,

    /// The file type to search (js, ts, php)
    #[arg(short = 't', long = "type")]
    pub file_type: String,
}

pub struct Config {
    query: String,
    file_path: String,
    file_type: FileType,
}

impl Config {
    pub fn new(
        query: String,
        file_path: String,
        file_type: String,
    ) -> Result<Config, &'static str> {
        Ok(Config {
            query,
            file_path,
            file_type: FileType::from_string(file_type)?,
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
    for line in search(config)? {
        println!("{}:{}:{}", line.file_path, line.line_number, line.text);
    }
    Ok(())
}

fn get_regexp_for_query(query: &str, file_type: FileType) -> Regex {
    let regexp_string = match file_type {
        FileType::JS => &format!(
            r"(\b(function|var|let|const|class)\s+{query}\b|\b{query}\([^)]*\)\s*\{{|\b{query}:)"
        ),
    };
    Regex::new(regexp_string).unwrap()
}

pub fn search(config: Config) -> Result<Vec<SearchResult>, Box<dyn Error>> {
    let contents = fs::read_to_string(&config.file_path)?;
    Ok(search_file(
        &config.query,
        &contents,
        config.file_type,
        &config.file_path,
    ))
}

fn search_file(
    query: &str,
    contents: &str,
    file_type: FileType,
    file_path: &str,
) -> Vec<SearchResult> {
    let re = get_regexp_for_query(query, file_type);

    contents
        .lines()
        .enumerate()
        .map(|(index, line)| SearchResult {
            file_path: String::from(file_path),
            line_number: index + 1,
            text: String::from(line),
        })
        .filter(|result| re.is_match(&result.text))
        .collect()
}
