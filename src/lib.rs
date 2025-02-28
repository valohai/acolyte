pub mod env;
pub mod stats;

use std::thread;
use tracing::info;

pub fn run_acolyte() {
    let stat_interval = env::get_stat_interval();
    loop {
        let num_cpus = stats::get_num_cpus().unwrap_or(1.0);
        let cpu_usage = stats::get_cpu_usage().unwrap_or(0.0);
        let mem_usage_kb = stats::get_memory_usage_kb().unwrap_or(0);
        let mem_total_kb = stats::get_memory_total_kb().unwrap_or(0);

        // scale the cpu usage by the number of cpus
        // so that 100% cpu usage on a 4 core machine is 4.0 etc.
        let normalized_cpu_usage = cpu_usage * num_cpus;

        let cpu_percent = cpu_usage * 100.0;
        let memory_percent = (mem_usage_kb as f64 / mem_total_kb as f64) * 100.0;

        let mem_usage_mb = mem_usage_kb / 1024;
        let mem_total_mb = mem_total_kb / 1024;

        info!(
            "CPU: {:.2} / {} ({:.2}%), Memory: {} MB / {} MB ({:.2}%)",
            normalized_cpu_usage, num_cpus, cpu_percent, mem_usage_mb, mem_total_mb, memory_percent
        );

        thread::sleep(stat_interval);
    }
}
