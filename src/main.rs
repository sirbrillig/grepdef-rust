use grepdef_rust::Config;
use std::env;
use std::process;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(exitcode::USAGE);
    });

    if let Err(err) = grepdef_rust::run(config) {
        eprintln!("{err}");
        process::exit(exitcode::USAGE);
    }
}
