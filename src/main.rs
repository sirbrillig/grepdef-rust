use grepdef_rust::Config;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(exitcode::USAGE);
    });
    if let Err(err) = grepdef_rust::run(config) {
        eprintln!("{err}");
        process::exit(exitcode::USAGE);
    }
}
