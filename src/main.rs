use std::io::{self, Write};

fn main() {
    let mut input = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();
        input = input.trim_end().to_string();
        let mut arguments = input.split(' ');
        let executable = arguments.next().unwrap();
        let arguments: Vec<&str> = arguments.collect();

        if executable.is_empty() {
            continue;
        } else if executable == "exit" {
            break;
        } else if executable == "echo" {
            println!(
                "{}",
                arguments
                    .iter()
                    .fold(String::new(), |acc, &arg| acc + arg + " ")
                    .trim_end()
            )
        } else {
            println!("{input}: command not found");
        }

        input.clear();
    }
}
