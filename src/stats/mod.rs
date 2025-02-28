mod proc;
mod utils;

use proc::ProcSource;
use std::io;

pub enum ResourceType {
    CPU,
    Memory,
}

pub struct CpuStats {
    pub num_cpus: f64,
    pub cpu_usage: f64,
}

pub struct MemoryStats {
    pub memory_usage_kb: u64,
    pub memory_total_kb: u64,
}

pub fn get_cpu_stats() -> io::Result<CpuStats> {
    let source = get_best_system_stats_source_for(ResourceType::CPU)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No available CPU stats source"))?;
    source.get_cpu_stats()
}

pub fn get_memory_stats() -> io::Result<MemoryStats> {
    let source = get_best_system_stats_source_for(ResourceType::Memory).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "No available memory stats source")
    })?;
    source.get_memory_stats()
}

pub fn get_best_system_stats_source_for(
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
    fn get_cpu_stats(&self) -> io::Result<CpuStats>;
    fn get_memory_stats(&self) -> io::Result<MemoryStats>;
    fn is_available_for(&self, resource_type: &ResourceType) -> bool;
}
