pub mod env;
pub mod stats;

use crate::stats::CpuUsageValue;
use std::thread;
use tracing::info;

pub fn run_acolyte() {
    let stat_interval = env::get_stat_interval();
    loop {
        let num_cpus = stats::get_num_cpus().unwrap_or(0.0); // TODO: handle zero
        let cpu_usage = stats::get_cpu_usage().unwrap_or(CpuUsageValue::FromCgroupV2(0.0));

        // scale the cpu usage by the number of cpus
        // so that 100% cpu usage on a 4 core machine is 4.0 etc.
        let normalized_cpu_usage = match cpu_usage {
            CpuUsageValue::FromCgroupV2(cgroup_usage) => cgroup_usage,
            CpuUsageValue::FromProc(proc_usage) => proc_usage * num_cpus,
        };

        let cpu_percent = match cpu_usage {
            CpuUsageValue::FromCgroupV2(cgroup_usage) => cgroup_usage / num_cpus * 100.0,
            CpuUsageValue::FromProc(proc_usage) => proc_usage * 100.0,
        };

        let mem_usage_kb = stats::get_memory_usage_kb().unwrap_or(0);
        let mem_total_kb = stats::get_memory_total_kb().unwrap_or(0);
        let mem_usage_mb = mem_usage_kb / 1024;
        let mem_total_mb = mem_total_kb / 1024;
        let mem_percent = (mem_usage_kb as f64 / mem_total_kb as f64) * 100.0;
        info!(
            "CPU: {:.2} / {} ({:.2}%), Memory: {} MB / {} MB ({:.2}%)",
            normalized_cpu_usage, num_cpus, cpu_percent, mem_usage_mb, mem_total_mb, mem_percent
        );

        if let Some(gpu_stats) = stats::get_gpu_stats() {
            let gpu_mem_usage_mb = gpu_stats.memory_usage_kb / 1024;
            let gpu_mem_total_mb = gpu_stats.memory_total_kb / 1024;
            info!(
                "GPU Usage: {} / {}, GPU Memory: {} MB / {} MB",
                gpu_stats.gpu_usage, gpu_stats.num_gpus, gpu_mem_usage_mb, gpu_mem_total_mb
            );
        }

        thread::sleep(stat_interval);
    }
}
