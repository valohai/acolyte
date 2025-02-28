use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn is_file_readable<P: AsRef<Path>>(path: P) -> bool {
    // NB: assumes that the file contains at least one byte of data
    let mut buffer = [0u8; 1];
    File::open(path)
        .and_then(|mut file| file.read(&mut buffer).map(|n| n > 0))
        .unwrap_or(false)
}
