use grepdef_rust::{search, FileType, SearchResult};

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
            file_path: String::from("test"),
            line_number: 4,
            text: String::from("function foo() {"),
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
            file_path: String::from("test"),
            line_number: 5,
            text: String::from("    let foobar = 'hello';"),
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
            file_path: String::from("test"),
            line_number: 5,
            text: String::from("    const foobar = 'hello';"),
        }],
        search(query, contents, FileType::JS, "test")
    )
}
