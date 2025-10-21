use std::io::{self, Write};
use std::fs;
use std::path::Path;


pub fn catfile(args: &[&str]) -> std::io::Result<()> {
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
              println!();
        }
    

    io::stdout().flush()?;
    Ok(())
}