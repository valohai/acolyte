pub mod env;
pub mod stats;
pub mod store;

use crate::stats::{
    CgroupVersion, CpuUsageValue, detect_cgroup_version, get_cgroup_v1_mount_points,
    get_cgroup_v2_mount_point,
};
use crate::store::StatsEntry;
use std::thread;
use tracing::{debug, error};

pub fn run_acolyte() {
    let stat_interval = env::get_stat_interval();
    let cgroup_detect = detect_cgroup_version("/proc/self/cgroup").ok();
    let v2_mount_point = match cgroup_detect {
        Some(CgroupVersion::V2) => get_cgroup_v2_mount_point("/proc/mounts").ok(),
        _ => None,
    };
    let v1_mount_points = match cgroup_detect {
        Some(CgroupVersion::V1) => get_cgroup_v1_mount_points("/proc/mounts").ok(),
        _ => None,
    };

    loop {
        let mut stats_entry = StatsEntry::new();

        let maybe_num_cpus = stats::get_num_cpus(
            cgroup_detect.clone(),
            v2_mount_point.clone(),
            v1_mount_points.clone(),
        );
        if let Some(num_cpus) = maybe_num_cpus {
            stats_entry.num_cpus = Some(num_cpus);
        }

        if let Some(cpu_usage) = stats::get_cpu_usage(
            cgroup_detect.clone(),
            v2_mount_point.clone(),
            v1_mount_points.clone(),
        ) {
            // scale the cpu usage by the number of cpus
            // so that 100% cpu usage on a 4 core machine is 4.0 etc.
            let normalized_cpu_usage = match cpu_usage {
                CpuUsageValue::FromCgroupV2(cgroup_usage) => Some(cgroup_usage),
                CpuUsageValue::FromCgroupV1(cgroup_usage) => Some(cgroup_usage),
                CpuUsageValue::FromProc(proc_usage) => {
                    // for the `procfs` values to report the number in the right format,
                    // we MUST know the number of cpus or the number will be misleading
                    if let Some(num_cpus) = maybe_num_cpus {
                        Some(proc_usage * num_cpus)
                    } else {
                        debug!("Failed to get number of CPUs, skipping procfs CPU usage");
                        None
                    }
                }
            };
            stats_entry.cpu_usage = normalized_cpu_usage;
        }

        if let Some(mem_usage_kb) = stats::get_memory_usage_kb(
            cgroup_detect.clone(),
            v2_mount_point.clone(),
            v1_mount_points.clone(),
        ) {
            stats_entry.memory_usage_kb = Some(mem_usage_kb);
        }

        if let Some(mem_total_kb) = stats::get_memory_total_kb(
            cgroup_detect.clone(),
            v2_mount_point.clone(),
            v1_mount_points.clone(),
        ) {
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
