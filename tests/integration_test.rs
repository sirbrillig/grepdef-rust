use grepdef_rust::{search, Args, Config, SearchResult};
use rstest::rstest;

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
    let args = Args {
        query,
        file_path,
        file_type: file_type_string,
        line_number: true,
    };
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
    let args = Args {
        query,
        file_path,
        file_type: file_type_string,
        line_number: true,
    };
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
    let args = Args {
        query,
        file_path,
        file_type: file_type_string,
        line_number: true,
    };
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
    let args = Args {
        query,
        file_path,
        file_type: file_type_string,
        line_number: true,
    };
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
    let args = Args {
        query,
        file_path,
        file_type: file_type_string,
        line_number: true,
    };
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
    let args = Args {
        query,
        file_path,
        file_type: file_type_string,
        line_number: true,
    };
    let config = Config::new(args).expect("Incorrect config for test");
    let actual = search(&config).expect("Search failed for test");
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}
