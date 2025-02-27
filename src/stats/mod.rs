use std::io;
use tracing::debug;
mod proc;

pub struct CpuStats {
    pub num_cpus: f64,
    pub cpu_usage: f64,
}

pub struct MemoryStats {
    pub memory_usage_kb: u64,
    pub memory_total_kb: u64,
}

pub fn get_cpu_stats() -> io::Result<CpuStats> {
    debug!("Using /proc for CPU stats");
    proc::get_cpu_stats()
}

pub fn get_memory_stats() -> io::Result<MemoryStats> {
    debug!("Using /proc for memory stats");
    proc::get_memory_stats()
}
