use std::{error::Error, fs};

use clap::ArgMatches;
use interpreter::{Interpreter, InterpreterConfig};
mod interpreter;

pub struct Config {
    pub input_program: String,
    pub duration_print: bool,
}

impl Config {
    pub fn new(arg_matches: ArgMatches) -> Result<Self, Box<dyn Error>> {
        let input_program = match arg_matches.value_of("input") {
            Some(input) => fs::read_to_string(input)?,
            None => arg_matches.value_of("rawInput").unwrap().to_string(),
        };

        let duration_print = arg_matches.is_present("showDuration");

        Ok(Self {
            input_program,
            duration_print,
        })
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut brain_fuck =
            match Interpreter::new(&self.input_program, InterpreterConfig::default()) {
                Ok(interpreter) => interpreter,
                Err(err) => return Err(Box::new(err)),
            };

        match brain_fuck.run() {
            Ok(result) => {
                if self.duration_print {
                    println!("Took {:?} to execute", result.execution_time());
                }
                Ok(())
            }
            Err(err) => Err(Box::new(err)),
        }
    }
}
