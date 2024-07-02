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
        Ok(Config {
            query,
            file_path,
            file_type: FileType::JS,
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
    let contents = fs::read_to_string(&config.file_path)?;
    for line in search(
        &config.query,
        &contents,
        config.file_type,
        &config.file_path,
    ) {
        println!("Found: {}", line.text);
    }
    Ok(())
}

pub fn search(
    query: &str,
    contents: &str,
    file_type: FileType,
    file_path: &str,
) -> Vec<SearchResult> {
    let mut results = Vec::new();

    let regexp_string = match file_type {
        FileType::JS => &format!(r"\b(function|let|const)\s+{query}\b"),
    };
    let re = Regex::new(regexp_string).unwrap();

    for (index, line) in contents.lines().enumerate() {
        if re.is_match(line) {
            let result = SearchResult {
                file_path: file_path.to_string(),
                line_number: index + 1,
                text: line.to_string(),
            };
            results.push(result)
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_returns_matching_js_function_line() {
        let query = "foo";
        let contents = "\
function bar() {
    console.log('bar');
}
function foo() {
    console.log('foo');
}
function foobar() {
    console.log('foobar');
}
function barfoo() {
    console.log('barfoo');
}
        ";
        assert_eq!(
            vec![SearchResult {
                file_path: "test".to_string(),
                line_number: 4,
                text: "function foo() {".to_string(),
            }],
            search(query, contents, FileType::JS, "test")
        )
    }

    #[test]
    fn search_returns_matching_js_let_line() {
        let query = "foobar";
        let contents = "\
function bar() {
    console.log('bar');
}
function foo() {
    let foobar = 'hello';
    console.log(foobar);
}
function barfoo() {
    console.log('barfoo');
}
            ";
        assert_eq!(
            vec![SearchResult {
                file_path: "test".to_string(),
                line_number: 5,
                text: "    let foobar = 'hello';".to_string(),
            }],
            search(query, contents, FileType::JS, "test")
        )
    }

    #[test]
    fn search_returns_matching_js_const_line() {
        let query = "foobar";
        let contents = "\
function bar() {
    console.log('bar');
}
function foo() {
    const foobar = 'hello';
    console.log(foobar);
}
function barfoo() {
    console.log('barfoo');
}
            ";
        assert_eq!(
            vec![SearchResult {
                file_path: "test".to_string(),
                line_number: 5,
                text: "    const foobar = 'hello';".to_string(),
            }],
            search(query, contents, FileType::JS, "test")
        )
    }
}
