use clap::Parser;
use grepdef_rust::Args;
use grepdef_rust::Config;
use std::process;

fn main() {
    let args = Args::parse();
    let config = Config::new(args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(exitcode::USAGE);
    });

    if let Err(err) = grepdef_rust::run(config) {
        eprintln!("{err}");
        process::exit(exitcode::USAGE);
    }
}
