use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn is_file_readable(path: &PathBuf) -> bool {
    let mut buffer = [0u8; 1];
    File::open(path)
        .and_then(|mut file| file.read(&mut buffer).map(|n| n > 0))
        .unwrap_or(false)
}
