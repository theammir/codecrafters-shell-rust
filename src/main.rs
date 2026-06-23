use std::io::{self, Write};

fn main() {
    let mut command = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut command).unwrap();
        command = command.trim_end().to_string();

        if command == "exit" {
            break;
        }

        println!("{command}: command not found");
        command.clear();
    }
}
