mod cgroup_v1;
mod cgroup_v2;
mod nvidia_smi;
mod paths;
mod proc;

use crate::stats::cgroup_v1::CgroupV1MountPoints;
pub use crate::stats::paths::{
    detect_cgroup_version, get_cgroup_v1_mount_points, get_cgroup_v2_mount_point,
};
use nvidia_smi::NvidiaSmiExecutor;
use proc::ProcSource;
use std::io;
use std::path::PathBuf;

pub enum ResourceType {
    NumCpus,
    CpuUsage,
    MemoryUsageKb,
    MemoryTotalKb,
}

// TODO: see if we could make this a bit simpler or give these a better name
pub enum CpuUsageValue {
    FromCgroupV2(f64), // normalized CPU usage i.e., 1.5 for one and a half CPUs busy
    FromCgroupV1(f64), // normalized CPU usage, like the V2 above
    FromProc(f64),     // fractional CPU usage i.e., 0.75 for 75% of all CPUs busy
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

pub fn get_num_cpus(
    cgroup_version: Option<CgroupVersion>,
    cgroup_v2_mount_point: Option<PathBuf>,
    cgroup_v1_mount_points: Option<CgroupV1MountPoints>,
) -> Option<f64> {
    if cgroup_version.as_ref().filter(|v| v.has_v2()).is_some() {
        if let Some(v2_mount_point) = cgroup_v2_mount_point {
            let source = cgroup_v2::CgroupV2Source::with_filesystem_reader_at(v2_mount_point);
            if let Ok(result) = source.get_num_cpus() {
                return Some(result);
            }
        }
    }

    if cgroup_version.as_ref().filter(|v| v.has_v1()).is_some() {
        if let Some(v1_mount_points) = cgroup_v1_mount_points {
            let source = cgroup_v1::CgroupV1Source::with_filesystem_reader_at(v1_mount_points);
            if let Ok(result) = source.get_num_cpus() {
                return Some(result);
            }
        }
    }

    let source = ProcSource::with_filesystem_reader_at(PathBuf::from("/proc"));
    source.get_num_cpus().ok()
}

pub fn get_cpu_usage(
    cgroup_version: Option<CgroupVersion>,
    cgroup_v2_mount_point: Option<PathBuf>,
    cgroup_v1_mount_points: Option<CgroupV1MountPoints>,
) -> Option<CpuUsageValue> {
    if cgroup_version.as_ref().filter(|v| v.has_v2()).is_some() {
        if let Some(v2_mount_point) = cgroup_v2_mount_point {
            let source = cgroup_v2::CgroupV2Source::with_filesystem_reader_at(v2_mount_point);
            if let Ok(result) = source.get_cpu_usage() {
                return Some(result);
            }
        }
    }

    if cgroup_version.as_ref().filter(|v| v.has_v1()).is_some() {
        if let Some(v1_mount_points) = cgroup_v1_mount_points {
            let source = cgroup_v1::CgroupV1Source::with_filesystem_reader_at(v1_mount_points);
            if let Ok(result) = source.get_cpu_usage() {
                return Some(result);
            }
        }
    }

    let source = ProcSource::with_filesystem_reader_at(PathBuf::from("/proc"));
    source.get_cpu_usage().ok()
}

pub fn get_memory_usage_kb(
    cgroup_version: Option<CgroupVersion>,
    cgroup_v2_mount_point: Option<PathBuf>,
    cgroup_v1_mount_points: Option<CgroupV1MountPoints>,
) -> Option<u64> {
    if cgroup_version.as_ref().filter(|v| v.has_v2()).is_some() {
        if let Some(v2_mount_point) = cgroup_v2_mount_point {
            let source = cgroup_v2::CgroupV2Source::with_filesystem_reader_at(v2_mount_point);
            if let Ok(result) = source.get_memory_usage_kb() {
                return Some(result);
            }
        }
    }

    if cgroup_version.as_ref().filter(|v| v.has_v1()).is_some() {
        if let Some(v1_mount_points) = cgroup_v1_mount_points {
            let source = cgroup_v1::CgroupV1Source::with_filesystem_reader_at(v1_mount_points);
            if let Ok(result) = source.get_memory_usage_kb() {
                return Some(result);
            }
        }
    }

    let source = ProcSource::with_filesystem_reader_at(PathBuf::from("/proc"));
    source.get_memory_usage_kb().ok()
}

pub fn get_memory_total_kb(
    cgroup_version: Option<CgroupVersion>,
    cgroup_v2_mount_point: Option<PathBuf>,
    cgroup_v1_mount_points: Option<CgroupV1MountPoints>,
) -> Option<u64> {
    if cgroup_version.as_ref().filter(|v| v.has_v2()).is_some() {
        if let Some(v2_mount_point) = cgroup_v2_mount_point {
            let source = cgroup_v2::CgroupV2Source::with_filesystem_reader_at(v2_mount_point);
            if let Ok(result) = source.get_memory_total_kb() {
                return Some(result);
            }
        }
    }

    if cgroup_version.as_ref().filter(|v| v.has_v1()).is_some() {
        if let Some(v1_mount_points) = cgroup_v1_mount_points {
            let source = cgroup_v1::CgroupV1Source::with_filesystem_reader_at(v1_mount_points);
            if let Ok(result) = source.get_memory_total_kb() {
                return Some(result);
            }
        }
    }

    let source = ProcSource::with_filesystem_reader_at(PathBuf::from("/proc"));
    source.get_memory_total_kb().ok()
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
