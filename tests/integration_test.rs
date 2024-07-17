use grepdef_rust::{search, Args, Config, SearchResult};
use rstest::rstest;

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
    }
}

#[rstest]
fn search_returns_matching_js_function_line() {
    let file_path = String::from("./tests/fixtures/js-fixture.js");
    let query = String::from("parseQuery");
    let line_number = 7;
    let file_type_string = String::from("js");
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(expected, actual);
}

#[rstest]
fn search_returns_matching_js_function_line_with_two_files() {
    let file_path = String::from("./tests/fixtures/js-fixture.js ./tests/fixtures/php-fixture.php");
    let query = String::from("parseQuery");
    let line_number = 7;
    let file_type_string = String::from("js");
    let expected = vec![SearchResult {
        file_path: String::from("./tests/fixtures/js-fixture.js"),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(expected, actual);
}

#[rstest]
fn search_returns_matching_js_function_line_with_one_file_one_directory_matching_on_directory() {
    let file_path = String::from("./tests/fixtures/ ./tests/fixtures/ignored-fixture.js");
    let query = String::from("parseQuery");
    let line_number = 6;
    let file_type_string = String::from("php");
    let expected = vec![SearchResult {
        file_path: String::from("./tests/fixtures/php-fixture.php"),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(expected, actual);
}

#[rstest]
fn search_returns_matching_js_function_line_with_one_file_one_directory_matching_on_file() {
    let file_path = String::from("./src/ ./tests/fixtures/js-fixture.js");
    let query = String::from("parseQuery");
    let line_number = 7;
    let file_type_string = String::from("js");
    let expected = vec![SearchResult {
        file_path: String::from("./tests/fixtures/js-fixture.js"),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(expected, actual);
}

#[rstest]
fn search_returns_matching_js_function_line_guessing_file_type() {
    let file_path = String::from("./tests/fixtures/js-fixture.js");
    let query = String::from("parseQuery");
    let line_number = 7;
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), None);
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(expected, actual);
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
    let file_path = String::from("./tests/fixtures/js-fixture.js");
    let query = String::from("parseQuery");
    let line_number = 7;
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(expected, actual);
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
fn search_returns_expected_line_number_js(
    #[case] query: String,
    #[case] file_type_string: String,
    #[case] line_number: usize,
) {
    let file_path = String::from("./tests/fixtures/js-fixture.js");
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let config = Config::new(args).expect("Search failed, invalid options");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(1, actual.len());
    let first_actual = actual.get(0).expect("Search failed for test");
    assert_eq!(line_number, first_actual.line_number);
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
fn search_returns_expected_line_number_jsx(
    #[case] query: String,
    #[case] file_type_string: String,
    #[case] line_number: usize,
) {
    let file_path = String::from("./tests/fixtures/jsx-fixture.jsx");
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let config = Config::new(args).expect("Search failed, invalid options");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(1, actual.len());
    let first_actual = actual.get(0).expect("Search failed for test");
    assert_eq!(line_number, first_actual.line_number);
}

#[rstest]
#[case(String::from("queryDbTS"), String::from("js"), 1)]
#[case(String::from("makeQueryTS"), String::from("js"), 4)]
#[case(String::from("parseQueryTS"), String::from("js"), 7)]
#[case(String::from("objectWithFunctionShorthandTS"), String::from("js"), 15)]
#[case(String::from("shorthandFunctionTS"), String::from("js"), 16)]
#[case(String::from("longhandFunctionTS"), String::from("js"), 25)]
#[case(String::from("longhandArrowFunctionTS"), String::from("js"), 34)]
#[case(String::from("longhandPropertyTS"), String::from("js"), 43)]
#[case(String::from("AnInterface"), String::from("js"), 59)]
#[case(String::from("AType"), String::from("js"), 63)]
#[case(String::from("TypeDefObject"), String::from("js"), 66)]
#[case(String::from("TypeDefSimple"), String::from("js"), 72)]
fn search_returns_expected_line_number_ts(
    #[case] query: String,
    #[case] file_type_string: String,
    #[case] line_number: usize,
) {
    let file_path = String::from("./tests/fixtures/ts-fixture.ts");
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let config = Config::new(args).expect("Search failed, invalid options");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(1, actual.len());
    let first_actual = actual.get(0).expect("Search failed for test");
    assert_eq!(line_number, first_actual.line_number);
}

#[rstest]
fn search_returns_matching_js_function_line_for_recursive() {
    let file_path = String::from("./tests");
    let query = String::from("parseQuery");
    let line_number = 7;
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
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}

#[rstest]
fn search_returns_matching_js_function_line_for_recursive_default_path() {
    let query = String::from("parseQuery");
    let line_number = 7;
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
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}

#[rstest]
fn search_returns_matching_ts_function_line_for_recursive() {
    let file_path = String::from("./tests");
    let query = String::from("parseQueryTS");
    let line_number = 7;
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
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}

#[rstest]
fn search_returns_matching_php_function_line() {
    let file_path = String::from("./tests/fixtures/php-fixture.php");
    let query = String::from("parseQuery");
    let line_number = 6;
    let file_type_string = String::from("php");
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(expected, actual);
}

#[rstest]
fn search_returns_matching_php_function_line_guessing_file_type_from_filename() {
    let file_path = String::from("./tests/fixtures/php-fixture.php");
    let query = String::from("parseQuery");
    let line_number = 6;
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), None);
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(expected, actual);
}

#[rstest]
fn search_returns_matching_php_function_line_guessing_file_type_from_directory() {
    let file_path = String::from("./tests/fixtures/only-php/other-php-fixture.php");
    let query = String::from("otherPhpFunction");
    let line_number = 3;
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function otherPhpFunction() {"),
    }];
    let args = make_args(query, Some(String::from("./tests/fixtures/only-php")), None);
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(expected, actual);
}

#[rstest]
#[case(String::from("queryDb"), String::from("php"), 2)]
#[case(String::from("parseQuery"), String::from("php"), 6)]
#[case(String::from("Foo"), String::from("php"), 11)]
#[case(String::from("Bar"), String::from("php"), 14)]
#[case(String::from("Zoom"), String::from("php"), 17)]
#[case(String::from("MyEnum"), String::from("php"), 20)]
#[case(String::from("doSomething"), String::from("php"), 24)]
#[case(String::from("doSomethingAbsolute"), String::from("php"), 26)]
fn search_returns_expected_line_number_php(
    #[case] query: String,
    #[case] file_type_string: String,
    #[case] line_number: usize,
) {
    let file_path = String::from("./tests/fixtures/php-fixture.php");
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let config = Config::new(args).expect("Search failed, invalid options");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(1, actual.len());
    let first_actual = actual.get(0).expect("Search failed for test");
    assert_eq!(line_number, first_actual.line_number);
}

#[rstest]
fn search_returns_matching_php_function_line_for_recursive() {
    let file_path = String::from("./tests");
    let query = String::from("parseQuery");
    let line_number = 6;
    let file_type_string = String::from("php");
    let expected = vec![SearchResult {
        file_path: String::from("./tests/fixtures/php-fixture.php"),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let args = make_args(query, Some(file_path), Some(file_type_string));
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}
