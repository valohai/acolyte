use std::thread;
use std::time::Duration;
use tracing::info;

pub fn run_acolyte() {
    loop {
        // imitate work...
        info!("Acolyte: For Ner'zhul!");
        thread::sleep(Duration::from_secs(1));
    }
}
