use clap::Parser;
use grepdef_rust::search;
use grepdef_rust::Args;
use grepdef_rust::Config;
use std::process;

fn main() {
    let config = Config::new(Args::parse()).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(exitcode::USAGE);
    });
    if config.no_color {
        colored::control::set_override(false);
    }
    match search(&config) {
        Ok(results) => {
            for line in results {
                println!("{}", line.to_grep());
            }
        }
        Err(err) => {
            eprintln!("{err}");
            process::exit(exitcode::USAGE);
        }
    };
}
