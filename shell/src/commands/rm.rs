use std::fs;
use std::io;
use std::path::Path;

pub fn rm(flags: &[String], args: &[String]) -> io::Result<()> {
    if args.is_empty() {
        eprintln!("rm: missing operand");
        return Ok(());
    }

    if !flags.is_empty(){
         if flags.len() != 1 || flags[0] != "r" {
        eprintln!("rm: only '-r' flag is supported");
        return Ok(());
    }
    }
    for target in args {
        let path = Path::new(target);

        if !path.exists() {
            eprintln!("rm: cannot remove '{}': No such file or directory", target);
            continue;
        }
        if flags.is_empty(){
            if path.is_dir(){
                eprintln!("rm : cannot remove {} : is a directory" , target);
                return Ok(());
            }
        }
        if path.is_dir() {
            if let Err(e) = fs::remove_dir_all(path) {
                eprintln!("rm: failed to remove directory '{}': {}", target, e);
            }
        } else if let Err(e) = fs::remove_file(path) {
            eprintln!("rm: failed to remove file '{}': {}", target, e);
        }
    }

    Ok(())
}
