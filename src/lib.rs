use regex::Regex;
use std::error::Error;
use std::fs;

pub struct Config {
    query: String,
    file_path: String,
    file_type: FileType,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        // Throw away first arg which is the name of the binary.
        args.next();

        let query = match args.next() {
            Some(qry) => qry,
            None => {
                return Err("You must provide a string to search for.");
            }
        };
        let file_path = match args.next() {
            Some(qry) => qry,
            None => {
                return Err("You must provide a file path after the search string.");
            }
        };
        let file_type_string = match args.next() {
            Some(qry) => qry,
            None => {
                return Err("You must provide a file type after the file path (TEMPORARY).");
            }
        };
        let file_type = match file_type_string.as_str() {
            "js" => FileType::JS,
            _ => {
                return Err("Invalid file type");
            }
        };
        Ok(Config {
            query,
            file_path,
            file_type,
        })
    }
}

pub enum FileType {
    JS,
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
        FileType::JS => &format!(r"\b(function|let|const)\s+{query}\b"),
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
