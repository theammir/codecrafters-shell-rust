use std::{
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
};

pub enum Command {
    Builtin(BuiltinCommandBody),
    Executable(ExecutableCommandBody),
}

pub enum BuiltinCommand {
    Exit,
    Echo,
    Type,
}

impl FromStr for BuiltinCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "exit" => Self::Exit,
            "echo" => Self::Echo,
            "type" => Self::Type,
            _ => Err(())?,
        })
    }
}

pub struct BuiltinCommandBody {
    pub cmd_type: BuiltinCommand,
    pub arguments: Vec<String>,
}

pub struct ExecutableCommandBody {
    pub executable_path: PathBuf,
    pub arguments: Vec<String>,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let arguments: Vec<String> = input.split(' ').map(String::from).collect();
        let executable = arguments.first().unwrap();

        let maybe_builtin = BuiltinCommand::from_str(executable).ok();
        if let Some(builtin) = maybe_builtin {
            Ok(Command::Builtin(BuiltinCommandBody {
                cmd_type: builtin,
                arguments,
            }))
        } else {
            Ok(Command::Executable(ExecutableCommandBody {
                executable_path: PathBuf::from(executable),
                arguments,
            }))
        }
    }
}

pub trait Execute {
    fn execute(&self) -> i8 {
        0
    }
}

impl Execute for BuiltinCommandBody {
    fn execute(&self) -> i8 {
        match self.cmd_type {
            BuiltinCommand::Exit => -127,
            BuiltinCommand::Echo => {
                println!(
                    "{}",
                    self.arguments
                        .iter()
                        .skip(1)
                        .fold(String::new(), |acc, arg| acc + arg + " ")
                        .trim_end()
                );
                0
            }
            BuiltinCommand::Type => {
                let Some(command_str) = self.arguments.get(1) else {
                    return 1;
                };
                let command = Command::from_str(command_str).unwrap();
                match command {
                    Command::Builtin(..) => println!("{command_str} is a shell built-in"),
                    Command::Executable(..) => println!("{command_str}: not found"),
                }
                0
            }
        }
    }
}

impl Execute for Command {
    fn execute(&self) -> i8 {
        match self {
            Self::Builtin(body) => body.execute(),
            Self::Executable(body) => {
                println!("{}: command not found", body.arguments.first().unwrap());
                0
            }
        }
    }
}

fn main() {
    let mut input = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();
        input = input.trim_end().to_string();

        if input.is_empty() {
            continue;
        }

        let command = input.parse::<Command>().unwrap();
        let code = command.execute();
        if code == -127 {
            break;
        }
        input.clear();
    }
}
