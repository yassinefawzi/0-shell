use std::fs;
use std::io;
use std::path::Path;

pub fn cpp(args: &[String]) -> io::Result<()> {
    let src = Path::new(&args[0]);
    let dst = Path::new(&args[1]);

    if src.is_dir() {
        eprintln!("cp: cannot copy directories");
        return Ok(());
    }

    if !src.exists() {
        eprintln!("cp: No such file or directory");
        return Ok(());
    }

    match fs::copy(src, dst) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("cp: failed to copy '{}': {}", src.display(), e);
            Ok(())
        }
    }
}
