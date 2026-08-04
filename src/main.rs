#![allow(
    clippy::cast_possible_truncation,
    reason = "allow truncating i32 status code to i8"
)]

use std::{
    borrow::Cow,
    ffi::OsStr,
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use regex::Regex;

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
    Cd,
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
            "cd" => Self::Cd,
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

impl Command {
    /// Parse a prompt or a part of it into an executable command.
    /// Caller assumes `input` is not empty, so panics here would mean that a real propmt can't be
    /// parsed correctly.
    fn parse(input: &str) -> Command {
        // https://regex101.com/r/rOKKjk/2
        let argument_split_pat =
            Regex::new(r#"(?:(?:'.+?'|".+?"|(?:\S+?(?:\\\S)*(?:\\\s)*))\s*?)+"#).unwrap();
        // https://regex101.com/r/YxybRt/2
        // i am literally going insane lmao
        let argument_parse_pat = Regex::new(
            r#"(?:(?<noq>(?:\\['"\s])?[^'"\s]+[^'"\s\\](?:\\['"\s])?)|'(?<singleq>[^'\r\n]*?)'|"(?<doubleq>[^"\r\n]*?)")"#,
        )
        .unwrap();
        let backslash_escape_pat = Regex::new(r"\\(.)").unwrap();

        // probably throwaway code, but it's funny how i can hide both the logics of parsing and the
        // logics of stitching it together behind confusing regular expression stuff.
        let arguments = argument_split_pat
            .find_iter(input)
            .map(|arg| {
                argument_parse_pat
                    .captures_iter(arg.as_str())
                    .map(|captures| {
                        if let Some(unquoted) = captures.name("noq").map(|m| m.as_str()) {
                            backslash_escape_pat.replace_all(unquoted, "$1")
                        } else if let Some(single_quoted) =
                            captures.name("singleq").map(|m| m.as_str())
                        {
                            Cow::Borrowed(single_quoted)
                        } else {
                            Cow::Borrowed(
                                captures
                                    .name("doubleq")
                                    .map(|m| m.as_str())
                                    .unwrap_or_default(),
                            )
                        }
                    })
                    .fold(String::new(), |mut arg, subarg| {
                        arg.push_str(subarg.as_ref());
                        arg
                    })
            })
            .collect::<Vec<_>>();
        let executable = arguments.first().unwrap();

        let maybe_builtin = BuiltinCommand::from_str(executable).ok();
        if let Some(builtin) = maybe_builtin {
            Command::Builtin(BuiltinCommandBody {
                cmd_type: builtin,
                arguments,
            })
        } else {
            Command::Executable(ExecutableCommandBody {
                executable_path: resolve_executable_path(Path::new(executable)),
                arguments,
            })
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
                let command = Command::parse(command_str);
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
            BuiltinCommand::Cd => {
                let new_pwd = self.arguments.get(1);
                let new_pwd_display = new_pwd.cloned().unwrap_or_else(|| String::from("~"));
                let new_pwd_path = {
                    let mut non_canonical = self
                        .arguments
                        .get(1)
                        .map_or_else(|| std::env::home_dir().unwrap_or_default(), PathBuf::from);
                    if non_canonical.as_os_str() == OsStr::new("~") {
                        non_canonical = std::env::home_dir().unwrap_or_default();
                    }
                    non_canonical.canonicalize()
                };
                match new_pwd_path {
                    Ok(new_pwd) if new_pwd.is_dir() => std::env::set_current_dir(new_pwd)
                        .err()
                        .map_or(0, |err| err.raw_os_error().unwrap_or(1))
                        as i8,
                    Ok(..) => {
                        println!("cd: {new_pwd_display}: Not a directory");
                        1
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                        println!("cd: {new_pwd_display}: No such file or directory");
                        1
                    }
                    Err(err) => {
                        println!("cd: {new_pwd_display}: {err}");
                        1
                    }
                }
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
                output.status.code().unwrap_or_default() as i8
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
        let trimmed = input.trim_end();

        if trimmed.is_empty() {
            continue;
        }

        let command = Command::parse(trimmed);
        let code = command.execute();
        if code == -127 {
            break;
        }
        drop(command);
        input.clear();
    }
}
