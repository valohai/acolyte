use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

pub fn read_first_line<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line)
}

pub fn read_all_lines<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    reader.lines().collect()
}

pub fn get_path_or_croak<'a>(
    path: &'a Option<PathBuf>,
    thing: &'static str,
) -> io::Result<&'a PathBuf> {
    path.as_ref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("{thing} file not found")))
}
