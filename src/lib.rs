use std::thread;
use tracing::info;

pub mod env;

pub fn run_acolyte() {
    let stat_interval = env::get_stat_interval();
    loop {
        // imitate work...
        info!("Acolyte: For Ner'zhul!");
        thread::sleep(stat_interval);
    }
}
