use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub fn read_first_line<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line)
}
