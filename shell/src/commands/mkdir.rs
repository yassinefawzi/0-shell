use std::fs;
use std::io;

pub fn mkdirr(args: &[String]) -> io::Result<()> {
    for dir in args {
        if let Err(e) = fs::create_dir(dir) {
            eprintln!("mkdir: cannot create directory '{}': {}", dir, e);
        }
    }

    Ok(())
}