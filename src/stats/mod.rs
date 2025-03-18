mod cgroup_v1;
mod cgroup_v2;
mod nvidia_smi;
mod paths;
mod proc;
mod utils;

use crate::stats::paths::{get_cgroup_v1_mount_points, get_cgroup_v2_mount_point};
use nvidia_smi::NvidiaSmiExecutor;
use paths::detect_cgroup_version;
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

#[derive(Debug, Eq, PartialEq)]
pub enum CgroupVersion {
    V1,
    V2,
}

pub fn get_num_cpus() -> Option<f64> {
    let source = get_best_system_stats_source_for(ResourceType::NumCpus)?;
    source.get_num_cpus().ok()
}

pub fn get_cpu_usage() -> Option<CpuUsageValue> {
    let source = get_best_system_stats_source_for(ResourceType::CpuUsage)?;
    source.get_cpu_usage().ok()
}

pub fn get_memory_usage_kb() -> Option<u64> {
    let source = get_best_system_stats_source_for(ResourceType::MemoryUsageKb)?;
    source.get_memory_usage_kb().ok()
}

pub fn get_memory_total_kb() -> Option<u64> {
    let source = get_best_system_stats_source_for(ResourceType::MemoryTotalKb)?;
    source.get_memory_total_kb().ok()
}

pub fn get_gpu_stats() -> Option<GpuStats> {
    // we only support NVIDIA GPUs for now so no need to check for other sources
    let executor = NvidiaSmiExecutor::new();
    nvidia_smi::get_gpu_stats(&executor).ok()
}

fn get_best_system_stats_source_for(
    resource_type: ResourceType,
) -> Option<Box<dyn SystemStatsSource>> {
    let detected_cgroup = detect_cgroup_version("/proc/self/cgroup").ok();

    if Some(CgroupVersion::V2) == detected_cgroup {
        if let Ok(v2_mount_point) = get_cgroup_v2_mount_point("/proc/mounts") {
            let source = cgroup_v2::CgroupV2Source::with_filesystem_reader_at(v2_mount_point);
            if source.is_available_for(&resource_type) {
                return Some(Box::new(source));
            };
        }
    }

    if Some(CgroupVersion::V1) == detected_cgroup {
        if let Ok(v1_mount_points) = get_cgroup_v1_mount_points("/proc/mounts") {
            let source = cgroup_v1::CgroupV1Source::with_filesystem_reader_at(v1_mount_points);
            if source.is_available_for(&resource_type) {
                return Some(Box::new(source));
            };
        }
    }

    let source = ProcSource::with_filesystem_reader_at(PathBuf::from("/proc"));
    if source.is_available_for(&resource_type) {
        return Some(Box::new(source));
    };

    None
}

pub trait SystemStatsSource {
    fn get_num_cpus(&self) -> io::Result<f64>;
    fn get_cpu_usage(&self) -> io::Result<CpuUsageValue>;
    fn get_memory_usage_kb(&self) -> io::Result<u64>;
    fn get_memory_total_kb(&self) -> io::Result<u64>;
    fn is_available_for(&self, resource_type: &ResourceType) -> bool;
}
