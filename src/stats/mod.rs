pub(crate) mod cgroup_v1;
pub(crate) mod cgroup_v2;
mod nvidia_smi;
mod paths;
pub(crate) mod proc;

pub use crate::stats::paths::{
    detect_cgroup_version, get_cgroup_v1_mount_points, get_cgroup_v2_mount_point,
};
use nvidia_smi::NvidiaSmiExecutor;
use std::io;
use tracing::debug;

// TODO: see if we could make this a bit simpler or give these a better name
pub enum CpuUsageValue {
    FromCgroupV2(f64), // normalized CPU usage i.e., 1.5 for one and a half CPUs busy
    FromCgroupV1(f64), // normalized CPU usage, like the V2 above
    FromProc(f64),     // fractional CPU usage i.e., 0.75 for 75% of all CPUs busy
}

impl CpuUsageValue {
    // scale the cpu usage by the number of cpus
    // so that 100% cpu usage on a 4 core machine is 4.0 etc.
    pub fn normalize(self, num_cpus: Option<f64>) -> Option<f64> {
        match self {
            CpuUsageValue::FromCgroupV2(cgroup_usage) => Some(cgroup_usage),
            CpuUsageValue::FromCgroupV1(cgroup_usage) => Some(cgroup_usage),
            CpuUsageValue::FromProc(proc_usage) => {
                // for the `procfs` values to report the number in the right format,
                // we MUST know the number of cpus or the number will be misleading
                if let Some(num_cpus) = num_cpus {
                    Some(proc_usage * num_cpus)
                } else {
                    debug!("Failed to get number of CPUs, skipping procfs CPU usage");
                    None
                }
            }
        }
    }
}

pub struct GpuStats {
    pub num_gpus: u32,        // N = number of GPUs
    pub gpu_usage: f64,       // normalized usage across all GPUs (0.0 - N.0)
    pub memory_usage_kb: u64, // sum of memory usage across all GPUs
    pub memory_total_kb: u64, // sum of total memory across all GPUs
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CgroupVersion {
    V1,
    V2,
    V1AndV2, // a "hybrid" setup; both V1 and V2 cgroups are present, and resources they control are mixed
}

impl CgroupVersion {
    pub fn has_v1(&self) -> bool {
        match self {
            Self::V1 | Self::V1AndV2 => true,
            Self::V2 => false,
        }
    }
    pub fn has_v2(&self) -> bool {
        match self {
            Self::V2 | Self::V1AndV2 => true,
            Self::V1 => false,
        }
    }
}

pub fn get_gpu_stats() -> Option<GpuStats> {
    // we only support NVIDIA GPUs for now so no need to check for other sources
    let executor = NvidiaSmiExecutor::new();
    nvidia_smi::get_gpu_stats(&executor).ok()
}

pub trait SystemStatsSource {
    fn get_num_cpus(&self) -> io::Result<f64>;
    fn get_cpu_usage(&self) -> io::Result<CpuUsageValue>;
    fn get_memory_usage_kb(&self) -> io::Result<u64>;
    fn get_memory_total_kb(&self) -> io::Result<u64>;
}
