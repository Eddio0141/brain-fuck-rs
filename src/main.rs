mod interpreter;

use std::env;

use interpreter::{Config, Interpreter};

fn main() {
    let mut args = env::args();
    args.next();

    // let cell_size = r#"Calculate the value 256 and test if it's zero
    // If the interpreter errors on overflow this is where it'll happen
    // ++++++++[>++++++++<-]>[<++++>-]
    // +<[>-<
    //     Not zero so multiply by 256 again to get 65536
    //     [>++++<-]>[<++++++++>-]<[>++++++++<-]
    //     +>[>
    //         # Print "32"
    //         ++++++++++[>+++++<-]>+.-.[-]<
    //     <[-]<->] <[>>
    //         # Print "16"
    //         +++++++[>+++++++<-]>.+++++.[-]<
    // <<-]] >[>
    //     # Print "8"
    //     ++++++++[>+++++++<-]>.[-]<
    // <-]<
    // # Print " bit cells\n"
    // +++++++++++[>+++>+++++++++>+++++++++>+<<<<-]>-.>-.+++++++.+++++++++++.<.
    // >>.++.+++++++..<-.>>-
    // Clean up used cells.
    // [[-]<]"#;

    let hello_world = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";

    let mut brain_fuck = match Interpreter::new(hello_world, Config::default()) {
        Ok(interpreter) => interpreter,
        Err(err) => panic!("{:?}", err),
    };

    match brain_fuck.run() {
        Ok(duration) => println!("Took {:#?} to execute.", duration),
        Err(err) => eprintln!("Error {}", err.to_string()),
    }
}
