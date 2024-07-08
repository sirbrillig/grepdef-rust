use grepdef_rust::{search, Config, SearchResult};
use rstest::rstest;

#[rstest]
fn search_returns_matching_js_function_line() {
    let file_path = String::from("./tests/js-fixture.js");
    let query = String::from("parseQuery");
    let line_number = 7;
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number,
        text: String::from("function parseQuery() {"),
    }];
    let config_args = [String::from("test run"), query, file_path].into_iter();
    let config = Config::build(config_args).expect("Incorrect config for test");
    let actual = search(config).expect("Search failed for test");
    assert_eq!(expected, actual)
}
