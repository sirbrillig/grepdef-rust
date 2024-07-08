use grepdef_rust::{search, Config, SearchResult};

#[test]
fn search_returns_matching_js_function_line() {
    let file_path = String::from("./tests/js-fixture.js");
    let expected = vec![SearchResult {
        file_path: file_path.clone(),
        line_number: 4,
        text: String::from("function foo() {"),
    }];
    let config_args = [String::from("test run"), String::from("foo"), file_path].into_iter();
    let config = Config::build(config_args).expect("Incorrect config for test");
    let actual = search(config).expect("Search failed for test");
    assert_eq!(expected, actual)
}
