pub mod env;
pub mod stats;

use std::thread;
use tracing::{debug, info};

pub fn run_acolyte() {
    let stat_interval = env::get_stat_interval();
    loop {
        match stats::get_cpu_stats() {
            Ok(cpu_stats) => {
                info!(
                    "CPU: {} of {} CPUs",
                    cpu_stats.cpu_usage, cpu_stats.num_cpus
                ); // TODO: save to file, don't print
            }
            Err(e) => {
                debug!("Failed to get CPU stats: {}", e);
            }
        }

        match stats::get_memory_stats() {
            Ok(mem_stats) => {
                info!(
                    "Memory: {} KB used of {} KB total ({:.2}%)",
                    mem_stats.memory_usage_kb,
                    mem_stats.memory_total_kb,
                    (mem_stats.memory_usage_kb as f64 / mem_stats.memory_total_kb as f64) * 100.0
                ); // TODO: save to file, don't print
            }
            Err(e) => {
                debug!("Failed to get memory stats: {}", e);
            }
        }

        thread::sleep(stat_interval);
    }
}
