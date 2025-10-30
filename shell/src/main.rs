mod commands;
mod parsing;
mod variables;

use commands::clear::*;
use commands::cp::*;
use commands::ls::*;
use commands::cat::*;
use commands::cd::*;
use commands::mkdir::*;
use commands::mv::*;
use commands::rm::*;
use parsing::split_save::*;
use std::env;
use std::io::Write;

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut var;

    loop {
        if let Ok(path) = env::current_dir() {
            print!("{}$ ", path.display());
        } else {
            print!("$ ");
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

        match var.command.as_str() {
            "exit" => break,

            "echo" => {
                if !var.args.is_empty() {
                    println!("{}", var.args.join(" "));
                } else {
                    println!("\n");
                }
            }

            "clear" => clearaw(),

            "pwd" => match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => eprintln!("Error getting current directory: {}", e),
            },
//a
            "cat" => {
                 if !var.flags.is_empty(){
                eprintln!("cat: -{} is not supported", var.flags.join(""));
                    continue;
                }
                let args: Vec<&str> = var.args.iter().map(|s| s.as_str()).collect();

                // Handle no arguments: read from stdin
                if args.is_empty() {
                    if let Err(e) = catfile(&[]) {
                        eprintln!("cat: {}", e);
                    }
                    continue;
                }

                // Handle each file argument
                for &file in &args {
                    if let Err(_) = catfile(&[file]) {
                        eprintln!("cat: {}: No such file or directory", file);
                    }
                }
            }

            "cd" => cdd(&var.args),

            "ls" => lss(&var.flags, &var.args),
            "mkdir" => {
                 if !var.flags.is_empty(){
                eprintln!("mkdir: -{} is not supported", var.flags.join(""));
                    continue;
                }
                if var.args.is_empty() {
                    eprintln!("mkdir: missing operand");
                    continue;
                }
                if let Err(e) = mkdirr(&var.args) {
                    eprintln!("mkdir: {}", e);
                }
            }

            "cp" => {
                 if !var.flags.is_empty(){
                eprintln!("cp: -{} is not supported", var.flags.join(""));
                    continue;
                }
                if var.args.len() < 2 {
                    eprintln!("cp: missing file operand");
                    continue;
                }
                if let Err(e) = cpp(&var.args) {
                    eprintln!("cp error: {}", e);
                }
            }

            "mv" => {
                if !var.flags.is_empty(){
                eprintln!("mv: -{} is not supported", var.flags.join(""));
                    continue;
                }
                if var.args.len() < 2 {
                    eprintln!("mv: missing file operand");
                    continue;
                }
                if let Err(e) = mvv(&var.args) {
                    eprintln!("mv error: {}", e);
                }
            }

            "rm" => {
                if let Err(e) = rm(&var.flags, &var.args) {
                    eprintln!("rm: {}", e);
                }
            }

            _ => println!("command not found: {}", var.command),
        }
    }
}

