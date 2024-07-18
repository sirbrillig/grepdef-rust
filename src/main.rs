use clap::Parser;
use grepdef_rust::Args;
use grepdef_rust::Searcher;
use std::process;

fn main() {
    let searcher = Searcher::new(Args::parse()).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(exitcode::USAGE);
    });
    match searcher.search() {
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
