mod parsing;
mod variables;
mod commands;

use commands::clear::*;
use commands::cp::*;
use commands::ls::*;
use commands::cat::*;
use commands::mv::*;
use commands::cd::*;
use std::io::{Write};
use std::env;
use parsing::split_save::*;
use variables::var::*;
use commands::mkdir::*;
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

        match var.command.as_str() {
            "exit" => break,
            "echo" => {
                if !var.args.is_empty(){
                    println!("{}" , var.args.join(" "));  
                }else{
                    println!("\n");
                }
            }
            "clear" => clearaw(),
          "pwd" => match env::current_dir() {
                 Ok(path) => println!("{}/", path.display()),
                Err(e) => eprintln!("Error getting current directory: {}", e),
            }
            "cat" => {
                if var.args.len() == 0{
                    eprintln!("error d cat f len");
                    continue ;
                }

             let args: Vec<&str> = var.args.iter().map(|s| s.as_str()).collect();
                        if let Err(e) = catfile(&args) {
                              eprintln!("cat {:?} : No such file or directory" ,  &args.join(" "));
                          }
            }
           "cd" => cdd(&var.args),
           //"ls" => lss(&var.flags, &var.args),
          "mkdir" => {
            if var.args.is_empty() {
                eprintln!("mkdir: missing operand");
                    continue;
             }
            if let Err(e) = mkdirr(&var.args) {
                 eprintln!("mkdir: {}", e);
                 }
            }
            "cp" => {
                 if var.args.len() < 2 {
                     eprintln!("cp: missing file operand");
                    continue;
                    }
                  if let Err(e) = cpp(&var.args) {
                        eprintln!("cp error: {}", e);
                    }
                }
                "mv"=>{
                        if var.args.len() < 2 {
                              eprintln!("mv: missing file operand");
                             continue;
                         }
                         if let Err(e) = mvv(&var.args) {
                        eprintln!("mv error: {}", e);
                    }
                }
            _ => println!("thawaa ? Command: {:?}", var),
        }
    }
}
