use std::io::{self, Write};
use std::fs;
use std::path::Path;

pub fn catfile(args: &[&str]) -> io::Result<()> {
    // If no args, behave like `cat -` (read stdin until Ctrl+C)
    if args.is_empty() {
        let stdin = io::stdin();
        let mut buffer = String::new();
        loop {
            match stdin.read_line(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    print!("{}", buffer);
                    buffer.clear();
                }
                Err(e) => return Err(e),
            }
        }
        return Ok(());
    }

    // Otherwise, print file contents
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
