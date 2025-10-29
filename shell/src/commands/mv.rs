use std::fs;
use std::path::Path;

pub fn mvv(args: &[String]) -> Result<(), String> {
    let sources = &args[..args.len() - 1];
    let dest = Path::new(&args[args.len() - 1]);

    if sources.len() > 1 {
        if !dest.is_dir() {
            return Err(format!("mv: target {}: is not a directory", dest.display()));
        }
    }

    for src in sources {
        let src_path = Path::new(src);

        if !src_path.exists() {
            eprintln!("mv: cannot stat {}: No such file or directory", src);
            continue;
        }
        
        let mut dest_path = dest.to_path_buf();
        if dest.is_dir() {
            if let Some(file_name) = src_path.file_name() {
                dest_path.push(file_name);
            }
        }

        if let Err(e) = fs::rename(&src_path, &dest_path) {
            eprintln!("mv: cannot move {} to {}: {}", src, dest_path.display(), e);
        }
    }

    Ok(())
}