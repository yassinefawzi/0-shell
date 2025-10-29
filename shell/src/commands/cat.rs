use std::io;
use std::fs;
use std::path::Path;

pub fn catfile(args: &[&str]) -> io::Result<()> {
    if args.is_empty() {
        let mut line = String::new();
        loop {
            line.clear();
            let n = io::stdin().read_line(&mut line)?;
            if n == 0 {
                break;
            }
            print!("{}", line);
        }
        return Ok(());
    }

    for &file in args {
        let path = Path::new(file);
        if path.is_dir() {
             eprintln!("'{}' is a directory", file);
        return Ok(());
        }

        let mut file = fs::File::open(path)?;
        io::copy(&mut file, &mut io::stdout())?;
    }

    Ok(())
}
