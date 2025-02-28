mod proc;
mod utils;

use proc::ProcSource;
use std::io;

pub enum ResourceType {
    NumCpus,
    CpuUsage,
    MemoryUsageKb,
    MemoryTotalKb,
}

pub fn get_num_cpus() -> Option<f64> {
    let source = get_best_system_stats_source_for(ResourceType::NumCpus)?;
    source.get_num_cpus().ok()
}

pub fn get_cpu_usage() -> Option<f64> {
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

fn get_best_system_stats_source_for(
    resource_type: ResourceType,
) -> Option<Box<dyn SystemStatsSource>> {
    // TODO: add cgroup v2 stat resolution here

    // TODO: add cgroup v1 stat resolution here

    let source = ProcSource::with_filesystem_reader_at("/proc");
    if source.is_available_for(&resource_type) {
        return Some(Box::new(source));
    };

    None
}

pub trait SystemStatsSource {
    fn get_num_cpus(&self) -> io::Result<f64>;
    fn get_cpu_usage(&self) -> io::Result<f64>;
    fn get_memory_usage_kb(&self) -> io::Result<u64>;
    fn get_memory_total_kb(&self) -> io::Result<u64>;
    fn is_available_for(&self, resource_type: &ResourceType) -> bool;
}
