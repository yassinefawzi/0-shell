use std::io::{self, Write};
use std::fs;
use std::path::Path;


pub fn Catfile(args: &[&str]) -> std::io::Result<()> {
    if args.is_empty() {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        io::copy(&mut reader, &mut io::stdout())?;
    } else {
        for &arg in args {
            let path = Path::new(arg);

            if path.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("'{}' is a directory", arg),
                ));
            }

            let mut file = fs::File::open(path)?;
            io::copy(&mut file, &mut io::stdout())?;
        }
    }

    io::stdout().flush()?;
    Ok(())
}