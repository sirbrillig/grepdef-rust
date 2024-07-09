use grepdef_rust::{search, Config, SearchResult};
use rstest::rstest;

#[rstest]
fn search_returns_matching_js_function_line() {
    let file_path = String::from("./tests/js-fixture.js");
    let query = String::from("parseQuery");
    let line_number = 7;
    let file_type_string = String::from("js");
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let config =
        Config::new(query, file_path, file_type_string, true).expect("Incorrect config for test");
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
fn search_returns_expected_line_number(
    #[case] query: String,
    #[case] file_type_string: String,
    #[case] line_number: usize,
) {
    let file_path = String::from("./tests/js-fixture.js");
    let config = Config::new(query, file_path, file_type_string, true)
        .expect("Search failed, invalid options");
    let actual = search(&config).expect("Search failed for test");
    assert_eq!(1, actual.len());
    let first_actual = actual.get(0).expect("Search failed for test");
    assert_eq!(line_number, first_actual.line_number);
}
