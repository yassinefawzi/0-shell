use std::env;
use std::path::Path;

pub fn cdd(args: &[String]) {
    if args.is_empty() {
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"));
        match home_dir {
            Ok(path) => {
                if let Err(e) = env::set_current_dir(Path::new(&path)) {
                    eprintln!("cd: {}", e);
                }
            }
            Err(_) => eprintln!("cd: cannot find home directory"),
        }
    } else if args[0] == "~" {
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"));
        match home_dir {
            Ok(path) => {
                if let Err(e) = env::set_current_dir(Path::new(&path)) {
                    eprintln!("cd: {}", e);
                }
            }
            Err(_) => eprintln!("cd: cannot find home directory"),
        }
    } else {
        let target = &args[0];
        if let Err(e) = env::set_current_dir(target) {
            eprintln!("cd: {}: {}", e, target);
        }
    }
}
