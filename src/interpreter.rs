use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    io::{self, Read, Stdin, Stdout, Write},
    num::Wrapping,
    time::{Duration, Instant},
};

pub struct Interpreter {
    commands: Vec<Command>,
    program_counter: usize,
    pointer: usize,
    cells: Vec<Wrapping<u8>>,
    config: InterpreterConfig,
    jump_cache: HashMap<usize, usize>,
}

impl Interpreter {
    pub fn new(commands: &str, config: InterpreterConfig) -> Result<Self, InterpreterError> {
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

        let jump_cache = {
            let mut depth: usize = 0;
            let jump_cache = commands
                .iter()
                .enumerate()
                .filter_map(|(i, cmd)| {
                    if matches!(cmd, Command::JumpPastIfZero) {
                        depth += 1;

                        let right_bracket = {
                            let mut depth_calc = depth;

                            commands[i + 1..]
                                .iter()
                                .enumerate()
                                .find(|(_, cmd)| {
                                    if matches!(cmd, Command::JumpBackIfNonZero) {
                                        depth_calc -= 1;
                                    } else if matches!(cmd, Command::JumpPastIfZero) {
                                        depth_calc += 1;
                                    }

                                    depth_calc < depth
                                })
                                .map(|(i, _)| i)
                        };

                        match right_bracket {
                            Some(right_bracket) => Some(Ok((i, right_bracket + i + 2))),
                            None => Some(Err(InterpreterError::RightBracketNotFound(i))),
                        }
                    } else if matches!(cmd, Command::JumpBackIfNonZero) {
                        let left_bracket = {
                            let mut depth_calc = depth;

                            commands[..i - 1]
                                .iter()
                                .enumerate()
                                .rev()
                                .find(|(_, cmd)| {
                                    if matches!(cmd, Command::JumpBackIfNonZero) {
                                        depth_calc += 1;
                                    } else if matches!(cmd, Command::JumpPastIfZero) {
                                        depth_calc -= 1;
                                    }

                                    depth_calc < depth
                                })
                                .map(|(i, _)| i)
                        };

                        depth -= 1;

                        match left_bracket {
                            Some(left_bracket) => Some(Ok((i, left_bracket + 1))),
                            None => Some(Err(InterpreterError::LeftBracketNotFound(i))),
                        }
                    } else {
                        None
                    }
                })
                .collect::<Result<HashMap<_, _>, InterpreterError>>();

            match jump_cache {
                Ok(jump_cache) => jump_cache,
                Err(err) => return Err(err),
            }
        };

        Ok(Self {
            commands,
            config,
            jump_cache,
            ..Default::default()
        })
    }

    pub fn run(&mut self) -> Result<InterpreterResult, InterpreterError> {
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
                Command::JumpPastIfZero => match self.jump_cache.get(&self.program_counter) {
                    Some(right_bracket) => {
                        if self.cells[self.pointer] == Wrapping(0) {
                            self.program_counter = *right_bracket;
                        } else {
                            self.program_counter += 1;
                        }
                    }
                    None => unreachable!(),
                },
                Command::JumpBackIfNonZero => match self.jump_cache.get(&self.program_counter) {
                    Some(left_bracket) => {
                        if self.cells[self.pointer] != Wrapping(0) {
                            self.program_counter = *left_bracket;
                        } else {
                            self.program_counter += 1;
                        }
                    }
                    None => unreachable!(),
                },
            }
            // println!("{:?} took {:#?}", command, before_exec.elapsed());
        }

        Ok(InterpreterResult {
            execution_time: before_exec.elapsed(),
        })
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
            jump_cache: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum InterpreterError {
    RightBracketNotFound(usize),
    LeftBracketNotFound(usize),
}

impl Error for InterpreterError {}

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

pub struct InterpreterConfig {
    stdout: Stdout,
    stdin: Stdin,
}

impl InterpreterConfig {
    /// Get a reference to the config's stdout.
    pub fn stdout(&self) -> &Stdout {
        &self.stdout
    }

    /// Get a reference to the config's stdin.
    pub fn stdin(&self) -> &Stdin {
        &self.stdin
    }
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            stdout: io::stdout(),
            stdin: io::stdin(),
        }
    }
}

pub struct InterpreterResult {
    execution_time: Duration,
}

impl InterpreterResult {
    /// Get a reference to the interpreter result's execution time.
    pub fn execution_time(&self) -> Duration {
        self.execution_time
    }
}
