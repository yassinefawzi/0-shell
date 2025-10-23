use std::fs;
use std::io;
use std::path::Path;

pub fn mvv(args: &[String]) -> io::Result<()> {
    let src = Path::new(&args[0]);
    let dst = Path::new(&args[1]);

    if !src.exists() {
        eprintln!("mv: cannot stat '{}': No such file or directory", src.display());
        return Ok(());
    }

    match fs::rename(src, dst) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("mv: failed to move '{}': {}", src.display(), e);
            Ok(())
        }
    }
}
