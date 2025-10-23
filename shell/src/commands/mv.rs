use std::fs;
use std::io;
use std::path::Path;

pub fn mvv(args: &[String]) -> io::Result<()> {
    if args.len() < 2 {
        eprintln!("mv: missing file operand");
        return Ok(());
    }

    let src = Path::new(&args[0]);
    let dst = Path::new(&args[1]);

    if !src.exists() {
        eprintln!("mv: cannot stat '{}': No such file or directory", src.display());
        return Ok(());
    }

    println!("Moving: {} -> {}", src.display(), dst.display());
    println!("Source is directory: {}", src.is_dir());
    println!("Destination exists: {}", dst.exists());
    if dst.exists() {
        println!("Destination is directory: {}", dst.is_dir());
    }

    // If destination exists and is a directory, move source into it
    let final_dst = if dst.exists() && dst.is_dir() {
        let file_name = src.file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("unknown"));
        dst.join(file_name)
    } else {
        dst.to_path_buf()
    };

    println!("Final destination: {}", final_dst.display());

    match fs::rename(src, &final_dst) {
        Ok(_) => {
            println!("Successfully moved {} to {}", src.display(), final_dst.display());
            Ok(())
        },
        Err(e) => {
            eprintln!("mv: failed to move '{}' to '{}': {}", src.display(), final_dst.display(), e);
            
            // Try cross-device move for directories
            if src.is_dir() {
                println!("Attempting cross-device move for directory...");
                match cross_device_move(src, &final_dst) {
                    Ok(_) => {
                        println!("Cross-device move successful");
                        Ok(())
                    },
                    Err(e) => {
                        eprintln!("mv: cross-device move failed: {}", e);
                        Ok(())
                    }
                }
            } else {
                Ok(())
            }
        }
    }
}

fn cross_device_move(src: &Path, dst: &Path) -> io::Result<()> {
    if src.is_dir() {
        copy_dir_all(src, dst)?;
        fs::remove_dir_all(src)?;
    } else {
        fs::copy(src, dst)?;
        fs::remove_file(src)?;
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}