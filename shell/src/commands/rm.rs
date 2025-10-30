use std::fs;
use std::io;
use std::path::Path;

pub fn rm(flags: &[String], args: &[String]) -> io::Result<()> {
    if args.is_empty() {
        eprintln!("rm: missing operand");
        return Ok(());
    }

    if !flags.is_empty() {
        if flags.len() != 1 || flags[0] != "r" {
            eprintln!("rm: only '-r' flag is supported");
            return Ok(());
        }
    }

    for target in args {
        let path = Path::new(target);

        // use symlink_metadata so it doesn't follow links
        let meta = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(_) => {
                eprintln!("rm: cannot remove '{}': No such file or directory", target);
                continue;
            }
        };

        let file_type = meta.file_type();

        if file_type.is_dir() && !file_type.is_symlink() {
            // directory (not a symlink)
            if flags.is_empty() {
                eprintln!("rm: cannot remove '{}': is a directory", target);
                continue;
            }

            if let Err(e) = fs::remove_dir_all(path) {
                eprintln!("rm: failed to remove directory '{}': {}", target, e);
            }
        } else {
            // file or symlink
            if let Err(e) = fs::remove_file(path) {
                eprintln!("rm: failed to remove file '{}': {}", target, e);
            }
        }
    }

    Ok(())
}
