mod parsing;
mod variables;

use std::io::{ Write };
use std::env;
use parsing::split_save::*;
use variables::var::*;

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut var = Var::new();

    loop {
        if let Ok(path) = env::current_dir() {
            print!("{}> ", path.display());
        } else {
            print!("?> ");
        }

        stdout.flush().unwrap();

        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => {
                println!("\nexit");
                break;
            }
            Ok(_) => {}
            Err(_) => {
                break;
            }
        }

        let command = input.trim().to_string();
        if command.is_empty() {
            continue;
        }
        var = split_save(command.clone());
        if var.command == "exit" {
            break;
        }
        println!("Command: {:?}", var);
    }
}
