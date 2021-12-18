use std::{
    fmt::Display,
    io::{self, Read, Stdin, Stdout, Write},
    num::Wrapping,
    time::{Duration, Instant},
};

pub struct Interpreter {
    commands: Vec<Command>,
    program_counter: usize,
    bracket_depth: usize,
    pointer: usize,
    cells: Vec<Wrapping<u8>>,
    config: Config,
}

impl Interpreter {
    pub fn new(commands: &str, config: Config) -> Self {
        let commands = commands
            .chars()
            .filter_map(|ch| match ch {
                '>' => Some(Command::PointerIncrease),
                '<' => Some(Command::PointerDecrease),
                '+' => Some(Command::IncreaseCell),
                '-' => Some(Command::DecreaseCell),
                '.' => Some(Command::OutputCell),
                ',' => Some(Command::InputCell),
                '[' => Some(Command::JumpPastIfZero),
                ']' => Some(Command::JumpBackIfNonZero),
                _ => None,
            })
            .collect::<Vec<_>>();

        Self {
            commands,
            config,
            ..Default::default()
        }
    }

    pub fn run(&mut self) -> Result<Duration, InterpreterError> {
        let mut stdout_handle = self.config.stdout().lock();
        let mut stdin_handle = self.config.stdin().lock();
        let mut stdin_buff = [0; 1];

        let before_exec = Instant::now();
        loop {
            let command = &self.commands.get(self.program_counter);
            let command = match command {
                Some(command) => command,
                None => break,
            };

            // println!(
            //     "program counter {}: executing {:?}. cell {} has a value {}",
            //     self.program_counter, command, self.pointer, self.cells[self.pointer]
            // );
            // let before_exec = Instant::now();
            match command {
                Command::PointerIncrease => {
                    self.pointer += 1;
                    self.program_counter += 1;
                }
                Command::PointerDecrease => {
                    self.pointer -= 1;
                    self.program_counter += 1;
                }
                Command::IncreaseCell => {
                    self.cells[self.pointer] += Wrapping(1);
                    self.program_counter += 1;
                }
                Command::DecreaseCell => {
                    self.cells[self.pointer] -= Wrapping(1);
                    self.program_counter += 1;
                }
                Command::OutputCell => {
                    stdout_handle
                        .write_all(&[self.cells[self.pointer].0])
                        .ok()
                        .unwrap();
                    self.program_counter += 1;
                }
                Command::InputCell => {
                    if stdin_handle.read(&mut stdin_buff).is_ok() {
                        self.cells[self.pointer] = Wrapping(stdin_buff[0]);
                    }
                    self.program_counter += 1;
                }
                Command::JumpPastIfZero => {
                    self.bracket_depth += 1;

                    let right_bracket = {
                        let brackets = &mut self.commands[self.program_counter + 1..]
                            .iter()
                            .enumerate()
                            .filter_map(|(i, cmd)| {
                                if matches!(cmd, Command::JumpBackIfNonZero)
                                    || matches!(cmd, Command::JumpPastIfZero)
                                {
                                    Some(i)
                                } else {
                                    None
                                }
                            });

                        let mut depth = self.bracket_depth;

                        brackets.find(|index| {
                            let cmd = &self.commands[*index + self.program_counter + 1];
                            if matches!(cmd, Command::JumpBackIfNonZero) {
                                depth -= 1;
                            } else if matches!(cmd, Command::JumpPastIfZero) {
                                depth += 1;
                            }
                            depth < self.bracket_depth
                        })
                    };

                    match right_bracket {
                        Some(mut right_bracket) => {
                            right_bracket += self.program_counter + 1;
                            if self.cells[self.pointer] == Wrapping(0) {
                                self.program_counter = right_bracket;
                            } else {
                                self.program_counter += 1;
                            }
                        }
                        None => {
                            return Err(InterpreterError::RightBracketNotFound(
                                self.program_counter,
                            ));
                        }
                    }
                }
                Command::JumpBackIfNonZero => {
                    let left_bracket = {
                        let brackets = &mut self.commands[..self.program_counter - 1]
                            .iter()
                            .enumerate()
                            .filter_map(|(i, cmd)| {
                                if matches!(cmd, Command::JumpBackIfNonZero)
                                    || matches!(cmd, Command::JumpPastIfZero)
                                {
                                    Some(i)
                                } else {
                                    None
                                }
                            });

                        let mut depth = self.bracket_depth;

                        brackets.rev().find(|index| {
                            let cmd = &self.commands[*index];
                            if matches!(cmd, Command::JumpBackIfNonZero) {
                                depth += 1;
                            } else if matches!(cmd, Command::JumpPastIfZero) {
                                depth -= 1;
                            }
                            depth < self.bracket_depth
                        })
                    };

                    match left_bracket {
                        Some(left_bracket) => {
                            if self.cells[self.pointer] != Wrapping(0) {
                                self.program_counter = left_bracket;
                            } else {
                                self.program_counter += 1;
                            }
                        }
                        None => {
                            return Err(InterpreterError::LeftBracketNotFound(
                                self.program_counter,
                            ));
                        }
                    }

                    self.bracket_depth -= 1;
                }
            }
            // println!("{:?} took {:#?}", command, before_exec.elapsed());
        }

        Ok(before_exec.elapsed())
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            pointer: 0,
            cells: vec![Wrapping(0); 30000],
            program_counter: 0,
            config: Default::default(),
            bracket_depth: 0,
        }
    }
}

pub enum InterpreterError {
    RightBracketNotFound(usize),
    LeftBracketNotFound(usize),
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::RightBracketNotFound(pos) => write!(
                f,
                "line: {}, right bracket not found to match the left bracket",
                pos
            ),
            InterpreterError::LeftBracketNotFound(pos) => write!(
                f,
                "line: {}, left bracket not found to match the right bracket",
                pos
            ),
        }
    }
}

#[derive(Debug)]
pub enum Command {
    PointerIncrease,
    PointerDecrease,
    IncreaseCell,
    DecreaseCell,
    OutputCell,
    InputCell,
    JumpPastIfZero,
    JumpBackIfNonZero,
}

pub struct Config {
    stdout: Stdout,
    stdin: Stdin,
}

impl Config {
    /// Get a reference to the config's stdout.
    pub fn stdout(&self) -> &Stdout {
        &self.stdout
    }

    /// Get a reference to the config's stdin.
    pub fn stdin(&self) -> &Stdin {
        &self.stdin
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stdout: io::stdout(),
            stdin: io::stdin(),
        }
    }
}
