use grepdef_rust::{search, Config, SearchResult};
use rstest::rstest;

#[rstest]
fn search_returns_matching_js_function_line() {
    let file_path = String::from("./tests/js-fixture.js");
    let query = String::from("parseQuery");
    let line_number = 7;
    let file_type = String::from("js");
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let config_args = [String::from("test run"), query, file_path, file_type].into_iter();
    let config = Config::build(config_args).expect("Incorrect config for test");
    let actual = search(config).expect("Search failed for test");
    assert_eq!(expected, actual);
}

#[rstest]
#[case(String::from("parseQuery"), String::from("js"), 7)]
fn search_returns_expected_line_number(
    #[case] query: String,
    #[case] file_type: String,
    #[case] line_number: usize,
) {
    let file_path = String::from("./tests/js-fixture.js");
    let config_args = [String::from("test run"), query, file_path, file_type].into_iter();
    let config = Config::build(config_args).expect("Incorrect config for test");
    let actual = search(config).expect("Search failed for test");
    let first_actual = actual.get(0).expect("Search failed for test");
    assert_eq!(line_number, first_actual.line_number);
}
