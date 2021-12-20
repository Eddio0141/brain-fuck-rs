use std::process;

use brain_fuck_rs::Config;
use clap::{App, Arg};

fn main() {
    let matches = App::new("brain-fuck-rs")
        .version("1.0")
        .author("yuu0141 <eddio0141@gmail.com>")
        .about("interpreter made in rust language for brainfuck")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input file for the interpreter")
                .required_unless("rawInput")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("rawInput")
                .short("raw")
                .long("rawInput")
                .value_name("INPUT")
                .takes_value(true)
                .case_insensitive(true)
                .help("Passes in raw brainfuck to the interpreter"),
        )
        .arg(
            Arg::with_name("showDuration")
                .short("d")
                .long("showDuration")
                .value_name("OPTIONS")
                .case_insensitive(true)
                .help("Prints the duration it took to run the interpreter"),
        )
        .get_matches();

    let config = Config::new(matches).unwrap_or_else(|err| {
        eprintln!("{}", err.to_string());
        process::exit(1);
    });

    if let Err(err) = config.run() {
        eprintln!("{}", err.to_string());
        process::exit(1);
    }
}
