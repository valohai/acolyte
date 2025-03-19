pub mod env;
pub mod stats;
pub mod store;
pub mod utils;

use crate::stats::cgroup_v1::CgroupV1Source;
use crate::stats::cgroup_v2::CgroupV2Source;
use crate::stats::proc::ProcSource;
use crate::stats::{
    CpuUsageValue, SystemStatsSource, detect_cgroup_version, get_cgroup_v1_mount_points,
    get_cgroup_v2_mount_point,
};
use crate::store::StatsEntry;
use std::path::PathBuf;
use std::thread;
use tracing::{debug, error};

pub fn run_acolyte() {
    let stat_interval = env::get_stat_interval();

    let sources = get_sources();

    loop {
        let mut stats_entry = StatsEntry::new();

        if let Some(num_cpus) = sources.iter().find_map(|source| source.get_num_cpus().ok()) {
            stats_entry.num_cpus = Some(num_cpus);
        }

        if let Some(cpu_usage) = sources
            .iter()
            .find_map(|source| source.get_cpu_usage().ok())
        {
            // scale the cpu usage by the number of cpus
            // so that 100% cpu usage on a 4 core machine is 4.0 etc.
            let normalized_cpu_usage = match cpu_usage {
                CpuUsageValue::FromCgroupV2(cgroup_usage) => Some(cgroup_usage),
                CpuUsageValue::FromCgroupV1(cgroup_usage) => Some(cgroup_usage),
                CpuUsageValue::FromProc(proc_usage) => {
                    // for the `procfs` values to report the number in the right format,
                    // we MUST know the number of cpus or the number will be misleading
                    if let Some(num_cpus) = stats_entry.num_cpus {
                        Some(proc_usage * num_cpus)
                    } else {
                        debug!("Failed to get number of CPUs, skipping procfs CPU usage");
                        None
                    }
                }
            };
            stats_entry.cpu_usage = normalized_cpu_usage;
        }

        if let Some(mem_usage_kb) = sources
            .iter()
            .find_map(|source| source.get_memory_usage_kb().ok())
        {
            stats_entry.memory_usage_kb = Some(mem_usage_kb);
        }

        if let Some(mem_total_kb) = sources
            .iter()
            .find_map(|source| source.get_memory_total_kb().ok())
        {
            stats_entry.memory_total_kb = Some(mem_total_kb);
        }

        if let Some(gpu_stats) = stats::get_gpu_stats() {
            stats_entry.num_gpus = Some(gpu_stats.num_gpus);
            stats_entry.gpu_usage = Some(gpu_stats.gpu_usage);
            stats_entry.gpu_memory_usage_kb = Some(gpu_stats.memory_usage_kb);
            stats_entry.gpu_memory_total_kb = Some(gpu_stats.memory_total_kb);
        }

        debug!("New stats entry: {:?}", stats_entry);
        if let Err(e) = store::write_stats_entry(stats_entry) {
            error!("Failed to write stats entry: {}", e);
        }

        thread::sleep(stat_interval);
    }
}

fn get_sources() -> Vec<Box<dyn SystemStatsSource>> {
    let mut sources: Vec<Box<dyn SystemStatsSource>> = vec![];
    let cgroup_version = detect_cgroup_version("/proc/self/cgroup").ok();

    if let Some(v2_mount_point) = cgroup_version
        .as_ref()
        .filter(|v| v.has_v2())
        .and_then(|_| get_cgroup_v2_mount_point("/proc/mounts").ok())
    {
        sources.push(Box::new(CgroupV2Source::with_filesystem_reader_at(
            v2_mount_point,
        )));
    }
    if let Some(v1_mount_points) = cgroup_version
        .as_ref()
        .filter(|v| v.has_v1())
        .and_then(|_| get_cgroup_v1_mount_points("/proc/mounts").ok())
    {
        sources.push(Box::new(CgroupV1Source::with_filesystem_reader_at(
            v1_mount_points,
        )));
    }
    sources.push(Box::new(ProcSource::with_filesystem_reader_at(
        PathBuf::from("/proc"),
    )));
    sources
}
