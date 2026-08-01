use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::is_executable::IsExecutable;

pub mod is_executable;

pub enum Command {
    Builtin(BuiltinCommandBody),
    Executable(ExecutableCommandBody),
}

pub enum BuiltinCommand {
    Exit,
    Echo,
    Type,
    Pwd,
}

impl FromStr for BuiltinCommand {
    type Err = ();

    // TODO: macro for a builtin command declaration maybe?
    // #[builtin("pwd")]
    // fn pwd_impl(arguments: Vec<String>) -> i8 {};
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "exit" => Self::Exit,
            "echo" => Self::Echo,
            "type" => Self::Type,
            "pwd" => Self::Pwd,
            _ => Err(())?,
        })
    }
}

// TODO: consider `OsString`s for everything

pub struct BuiltinCommandBody {
    pub cmd_type: BuiltinCommand,
    pub arguments: Vec<String>,
}

pub struct ExecutableCommandBody {
    /// Path to the executable, resolved from PATH, system root or the current directory.
    /// Intentionally not canonicalized, so might be relative to cwd or contain symlinks.
    /// `None` if the actual executable wasn't found anywhere.
    pub executable_path: Option<PathBuf>,
    pub arguments: Vec<String>,
}

fn resolve_executable_path(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() || path.starts_with("./") {
        if path.is_executable() {
            return Some(path.to_owned());
        }
        return None;
    }

    let executable_name = path.file_name()?;
    let path_env = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::env::split_paths(&path_env);

    paths
        .filter(|p| p.is_dir() && p.is_absolute())
        .find_map(|p_dir| {
            std::fs::read_dir(p_dir).ok()?.find_map(|file| {
                let path = file.ok()?.path();
                (path.is_file()
                    && path.is_executable()
                    && path.file_name().is_some_and(|n| n == executable_name))
                .then_some(path)
            })
        })
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
                executable_path: resolve_executable_path(Path::new(executable)),
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
                    Command::Builtin(..) => println!("{command_str} is a shell builtin"),
                    Command::Executable(body) => match body.executable_path {
                        Some(path) => println!("{command_str} is {}", path.display()),
                        None => println!("{command_str}: not found"),
                    },
                }
                0
            }
            BuiltinCommand::Pwd => {
                let Ok(pwd) = std::env::current_dir() else {
                    return 1;
                };
                println!("{}", pwd.display());
                0
            }
        }
    }
}

impl Execute for Command {
    fn execute(&self) -> i8 {
        match self {
            Self::Builtin(body) => body.execute(),
            Self::Executable(ExecutableCommandBody {
                executable_path: None,
                arguments,
            }) => {
                println!("{}: command not found", arguments.first().unwrap());
                0
            }
            Self::Executable(ExecutableCommandBody {
                executable_path: Some(executable_path),
                arguments,
            }) => {
                let mut command = std::process::Command::new(executable_path);
                command.args(arguments.iter().skip(1));
                let handle = command
                    .spawn()
                    .unwrap_or_else(|_| panic!("failed to spawn \"{}\"", arguments[0]));
                let output = handle.wait_with_output().unwrap();

                // NOTE: doesn't handle signal termination
                #[allow(clippy::cast_possible_truncation)] // unix behaviour anyway
                let code = output.status.code().unwrap_or_default() as i8;
                code
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
