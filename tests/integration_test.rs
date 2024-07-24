use grepdef::{Args, FileType, SearchResult, Searcher};
use rstest::rstest;
use std::num::NonZero;

fn make_args(query: String, file_path: Option<String>, file_type_string: Option<String>) -> Args {
    Args {
        query,
        file_path: match file_path {
            Some(file_path) => Some(file_path.split_whitespace().map(String::from).collect()),
            None => None,
        },
        file_type: file_type_string,
        line_number: true,
        search_method: None,
        debug: false,
        no_color: false,
        threads: None,
    }
}

fn do_search(args: Args) -> Vec<SearchResult> {
    let searcher = Searcher::new(args).unwrap();
    searcher.search().expect("Search failed for test")
}

fn get_default_fixture_for_file_type_string(file_type_string: &str) -> Result<String, String> {
    match file_type_string {
        "js" => Ok(get_default_fixture_for_file_type(FileType::JS)),
        "ts" => Ok(String::from("./tests/fixtures/ts-fixture.ts")),
        "jsx" => Ok(String::from("./tests/fixtures/jsx-fixture.jsx")),
        "tsx" => Ok(String::from("./tests/fixtures/jsx-fixture.tsx")),
        "php" => Ok(get_default_fixture_for_file_type(FileType::PHP)),
        "rs" => Ok(get_default_fixture_for_file_type(FileType::RS)),
        _ => {
            return Err(format!(
                "No fixture found for file type '{}'",
                file_type_string
            ))
        }
    }
}

fn get_default_fixture_for_file_type(file_type: FileType) -> String {
    match file_type {
        FileType::JS => String::from("./tests/fixtures/js-fixture.js"),
        FileType::PHP => String::from("./tests/fixtures/php-fixture.php"),
        FileType::RS => String::from("./tests/fixtures/rs-fixture.rs"),
    }
}

fn get_expected_text_line_for_test_search(
    file_type_string: &str,
) -> Result<(String, usize), String> {
    match file_type_string {
        "js" => Ok((String::from("function parseQuery() {"), 7)),
        "ts" => Ok((String::from("function parseQueryTS() {"), 7)),
        "jsx" => Ok((String::from("function parseQuery() {"), 6)),
        "tsx" => Ok((String::from("function parseQueryTS() {"), 7)),
        "php" => Ok((String::from("function parseQuery() {"), 6)),
        "rs" => Ok((String::from("pub fn query_db() -> bool {}"), 1)),
        _ => {
            return Err(format!(
                "No expected text found for file type '{}'",
                file_type_string
            ))
        }
    }
}

#[rstest]
fn search_returns_matching_js_function_line_with_args_new() {
    let file_path = get_default_fixture_for_file_type(FileType::JS);
    let query = String::from("parseQuery");
    let line_number = Some(7);
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let actual = do_search(Args::new(
        query,
        Some("js".into()),
        Some(vec![file_path]),
        true,
    ));
    assert_eq!(expected, actual);
}

#[rstest]
fn search_returns_nothing_for_no_results() {
    let file_path = get_default_fixture_for_file_type(FileType::JS);
    let query = String::from("thisFunctionDoesNotExist");
    let file_type_string = String::from("js");
    let expected: Vec<SearchResult> = vec![];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, do_search(args));
}

#[rstest]
fn search_returns_matching_js_function_line_with_one_thread() {
    let file_path = get_default_fixture_for_file_type(FileType::JS);
    let query = String::from("thisFunctionDoesNotExist");
    let file_type_string = String::from("js");
    let expected: Vec<SearchResult> = vec![];
    let mut args = make_args(query, Some(file_path), Some(file_type_string));
    args.threads = Some(NonZero::new(1).unwrap());
    assert_eq!(expected, do_search(args));
}

#[rstest]
fn search_returns_matching_js_function_line_with_two_files() {
    let file_path = String::from("./tests/fixtures/js-fixture.js ./tests/fixtures/php-fixture.php");
    let query = String::from("parseQuery");
    let line_number = Some(7);
    let file_type_string = String::from("js");
    let expected = vec![SearchResult {
        file_path: String::from("./tests/fixtures/js-fixture.js"),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, do_search(args));
}

#[rstest]
fn search_returns_matching_js_function_line_with_one_file_one_directory_matching_on_directory() {
    let file_path = String::from("./tests/fixtures/ ./tests/fixtures/ignored-fixture.js");
    let query = String::from("parseQuery");
    let line_number = Some(6);
    let file_type_string = String::from("php");
    let expected = vec![SearchResult {
        file_path: String::from("./tests/fixtures/php-fixture.php"),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, do_search(args));
}

#[rstest]
fn search_returns_matching_js_function_line_with_one_file_one_directory_matching_on_file() {
    let file_path = String::from("./src/ ./tests/fixtures/js-fixture.js");
    let query = String::from("parseQuery");
    let line_number = Some(7);
    let file_type_string = String::from("js");
    let expected = vec![SearchResult {
        file_path: String::from("./tests/fixtures/js-fixture.js"),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, do_search(args));
}

#[rstest]
#[case(String::from("parseQuery"), String::from("js"))]
#[case(String::from("parseQuery"), String::from("php"))]
#[case(String::from("query_db"), String::from("rs"))]
fn search_returns_matching_function_line_guessing_file_type_from_file_name(
    #[case] query: String,
    #[case] file_type_string: String,
) {
    let file_path = get_default_fixture_for_file_type_string(file_type_string.as_str()).unwrap();
    let (text, line_number) =
        get_expected_text_line_for_test_search(file_type_string.as_str()).unwrap();
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number: Some(line_number),
        text,
    }];
    let args = make_args(query, Some(file_path), None);
    assert_eq!(expected, do_search(args));
}

#[rstest]
#[case(String::from("parseQuery"), String::from("js"))]
#[case(String::from("parseQuery"), String::from("php"))]
#[case(String::from("query_db"), String::from("rs"))]
fn search_returns_matching_function_line(#[case] query: String, #[case] file_type_string: String) {
    let file_path = get_default_fixture_for_file_type_string(file_type_string.as_str()).unwrap();
    let (text, line_number) =
        get_expected_text_line_for_test_search(file_type_string.as_str()).unwrap();
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number: Some(line_number),
        text,
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, do_search(args));
}

#[rstest]
#[case(String::from("js"))]
#[case(String::from("ts"))]
#[case(String::from("jsx"))]
#[case(String::from("tsx"))]
#[case(String::from("javascript"))]
#[case(String::from("javascript.jsx"))]
#[case(String::from("javascriptreact"))]
#[case(String::from("typescript"))]
#[case(String::from("typescript.tsx"))]
#[case(String::from("typescriptreact"))]
fn search_returns_matching_js_function_line_with_filetype_alias(#[case] file_type_string: String) {
    let file_path = get_default_fixture_for_file_type(FileType::JS);
    let query = String::from("parseQuery");
    let line_number = Some(7);
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, do_search(args));
}

#[rstest]
#[case(String::from("queryDb"), String::from("js"), 1)]
#[case(String::from("makeQuery"), String::from("js"), 4)]
#[case(String::from("parseQuery"), String::from("js"), 7)]
#[case(String::from("objectWithFunctionShorthand"), String::from("js"), 15)]
#[case(String::from("shorthandFunction"), String::from("js"), 16)]
#[case(String::from("longhandFunction"), String::from("js"), 25)]
#[case(String::from("longhandArrowFunction"), String::from("js"), 34)]
#[case(String::from("longhandProperty"), String::from("js"), 43)]
#[case(String::from("queryDb"), String::from("jsx"), 1)]
#[case(String::from("makeQuery"), String::from("jsx"), 4)]
#[case(String::from("parseQuery"), String::from("jsx"), 7)]
#[case(String::from("objectWithFunctionShorthand"), String::from("jsx"), 15)]
#[case(String::from("shorthandFunction"), String::from("jsx"), 16)]
#[case(String::from("longhandFunction"), String::from("jsx"), 25)]
#[case(String::from("longhandArrowFunction"), String::from("jsx"), 34)]
#[case(String::from("longhandProperty"), String::from("jsx"), 43)]
#[case(String::from("queryDbTS"), String::from("ts"), 1)]
#[case(String::from("makeQueryTS"), String::from("ts"), 4)]
#[case(String::from("parseQueryTS"), String::from("ts"), 7)]
#[case(String::from("objectWithFunctionShorthandTS"), String::from("ts"), 15)]
#[case(String::from("shorthandFunctionTS"), String::from("ts"), 16)]
#[case(String::from("longhandFunctionTS"), String::from("ts"), 25)]
#[case(String::from("longhandArrowFunctionTS"), String::from("ts"), 34)]
#[case(String::from("longhandPropertyTS"), String::from("ts"), 43)]
#[case(String::from("AnInterface"), String::from("ts"), 59)]
#[case(String::from("AType"), String::from("ts"), 63)]
#[case(String::from("TypeDefObject"), String::from("ts"), 66)]
#[case(String::from("TypeDefSimple"), String::from("ts"), 72)]
#[case(String::from("queryDb"), String::from("php"), 2)]
#[case(String::from("parseQuery"), String::from("php"), 6)]
#[case(String::from("Foo"), String::from("php"), 11)]
#[case(String::from("Bar"), String::from("php"), 14)]
#[case(String::from("Zoom"), String::from("php"), 17)]
#[case(String::from("MyEnum"), String::from("php"), 20)]
#[case(String::from("doSomething"), String::from("php"), 24)]
#[case(String::from("doSomethingAbsolute"), String::from("php"), 26)]
#[case(String::from("query_db"), String::from("rs"), 1)]
#[case(String::from("public_func"), String::from("rs"), 6)]
#[case(String::from("Wrapper"), String::from("rs"), 4)]
#[case(String::from("ContainerWithoutBlock"), String::from("rs"), 9)]
#[case(String::from("ContainerWithBlock"), String::from("rs"), 11)]
#[case(String::from("FileType"), String::from("rs"), 19)]
#[case(String::from("search_file"), String::from("rs"), 29)]
fn search_returns_expected_line_number_for_file_type(
    #[case] query: String,
    #[case] file_type_string: String,
    #[case] line_number: usize,
) {
    let file_path = get_default_fixture_for_file_type_string(file_type_string.as_str()).unwrap();
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let actual = do_search(args);
    assert_eq!(1, actual.len());
    let first_actual = actual.get(0).expect("Search failed for test");
    assert_eq!(line_number, first_actual.line_number.unwrap());
}

#[rstest]
fn search_returns_matching_js_function_line_for_recursive() {
    let file_path = String::from("./tests");
    let query = String::from("parseQuery");
    let line_number = Some(7);
    let file_type_string = String::from("js");
    let expected = vec![
        SearchResult {
            file_path: String::from("./tests/fixtures/js-fixture.js"),
            line_number,
            text: String::from("function parseQuery() {"),
        },
        SearchResult {
            file_path: String::from("./tests/fixtures/jsx-fixture.jsx"),
            line_number,
            text: String::from("function parseQuery() {"),
        },
    ];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let actual = do_search(args);
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}

#[rstest]
fn search_returns_matching_js_function_line_for_recursive_default_path() {
    let query = String::from("parseQuery");
    let line_number = Some(7);
    let file_type_string = String::from("js");
    let expected = vec![
        SearchResult {
            file_path: String::from("./tests/fixtures/js-fixture.js"),
            line_number,
            text: String::from("function parseQuery() {"),
        },
        SearchResult {
            file_path: String::from("./tests/fixtures/jsx-fixture.jsx"),
            line_number,
            text: String::from("function parseQuery() {"),
        },
    ];
    let args = make_args(query, None, Some(file_type_string));
    let actual = do_search(args);
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}

#[rstest]
fn search_returns_matching_ts_function_line_for_recursive() {
    let file_path = String::from("./tests");
    let query = String::from("parseQueryTS");
    let line_number = Some(7);
    // Note that the type is still JS
    let file_type_string = String::from("js");
    let expected = vec![
        SearchResult {
            file_path: String::from("./tests/fixtures/ts-fixture.ts"),
            line_number,
            text: String::from("function parseQueryTS(): string {"),
        },
        SearchResult {
            file_path: String::from("./tests/fixtures/tsx-fixture.tsx"),
            line_number,
            text: String::from("function parseQueryTS(): string {"),
        },
    ];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let actual = do_search(args);
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}

#[rstest]
fn search_returns_matching_php_function_line_guessing_file_type_from_directory() {
    let file_path = String::from("./tests/fixtures/only-php/other-php-fixture.php");
    let query = String::from("otherPhpFunction");
    let line_number = Some(3);
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function otherPhpFunction() {"),
    }];
    let args = make_args(query, Some(String::from("./tests/fixtures/only-php")), None);
    assert_eq!(expected, do_search(args));
}

#[rstest]
fn search_returns_matching_php_function_line_for_recursive() {
    let file_path = String::from("./tests");
    let query = String::from("parseQuery");
    let line_number = Some(6);
    let file_type_string = String::from("php");
    let expected = vec![SearchResult {
        file_path: String::from("./tests/fixtures/php-fixture.php"),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let actual = do_search(args);
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}

#[rstest]
fn search_returns_matching_rs_function_line_for_recursive() {
    let file_path = String::from("./tests/fixtures");
    let query = String::from("query_db");
    let line_number = Some(1);
    let file_type_string = String::from("rs");
    let expected = vec![SearchResult {
        file_path: String::from("./tests/fixtures/rs-fixture.rs"),
        line_number,
        text: String::from("pub fn query_db() -> bool {}"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let actual = do_search(args);
    println!("{:?}", actual);
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}
