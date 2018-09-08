extern crate snake;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    // parse config from arguments
    let config = snake::Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {}.", err);
        process::exit(1);
    });
    // run the game
    if let Err(err) = snake::run(config) {
        eprintln!("Application error: {}.", err);
        process::exit(1);
    };
}
