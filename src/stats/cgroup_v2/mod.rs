use crate::stats::{CpuUsageValue, SystemStatsSource};
mod cpu_usage;
mod memory_current;
mod memory_max;
mod num_cpus;
use crate::utils::{read_all_lines, read_first_line};
#[cfg(test)]
use mockall::automock;
use std::io::{self};
use std::path::PathBuf;
use std::time::Duration;

pub struct CgroupV2Source<P: CgroupV2Provider> {
    provider: P,
}

impl<P: CgroupV2Provider> CgroupV2Source<P> {
    fn new(provider: P) -> Self {
        Self { provider }
    }
}

impl CgroupV2Source<CgroupV2FilesystemReader> {
    pub fn with_filesystem_reader_at(path: PathBuf) -> Self {
        Self::new(CgroupV2FilesystemReader::new(path))
    }
}

impl<P: CgroupV2Provider> SystemStatsSource for CgroupV2Source<P> {
    fn get_num_cpus(&self) -> io::Result<f64> {
        num_cpus::get_num_cpus(&self.provider)
    }

    fn get_cpu_usage(&self, sample_interval: Duration) -> io::Result<CpuUsageValue> {
        cpu_usage::get_cpu_usage(&self.provider, sample_interval)
    }

    fn get_memory_usage_kb(&self) -> io::Result<u64> {
        memory_current::get_memory_current_kb(&self.provider)
    }

    fn get_memory_total_kb(&self) -> io::Result<u64> {
        memory_max::get_memory_max_kb(&self.provider)
    }
}

pub struct CgroupV2FilesystemReader {
    cpu_max_path: PathBuf,
    cpu_stat_path: PathBuf,
    mem_current_path: PathBuf,
    mem_max_path: PathBuf,
}

impl CgroupV2FilesystemReader {
    fn new(cgroup_v2_path: PathBuf) -> Self {
        Self {
            cpu_max_path: cgroup_v2_path.join("cpu.max"),
            cpu_stat_path: cgroup_v2_path.join("cpu.stat"),
            mem_current_path: cgroup_v2_path.join("memory.current"),
            mem_max_path: cgroup_v2_path.join("memory.max"),
        }
    }
}

impl CgroupV2Provider for CgroupV2FilesystemReader {
    fn get_cgroup_v2_cpu_stat(&self) -> io::Result<Vec<String>> {
        read_all_lines(&self.cpu_stat_path)
    }

    fn get_cgroup_v2_cpu_max(&self) -> io::Result<String> {
        read_first_line(&self.cpu_max_path)
    }

    fn get_cgroup_v2_memory_current(&self) -> io::Result<String> {
        read_first_line(&self.mem_current_path)
    }

    fn get_cgroup_v2_memory_max(&self) -> io::Result<String> {
        read_first_line(&self.mem_max_path)
    }
}

#[cfg_attr(test, automock)]
pub trait CgroupV2Provider {
    fn get_cgroup_v2_cpu_stat(&self) -> io::Result<Vec<String>>;
    fn get_cgroup_v2_cpu_max(&self) -> io::Result<String>;
    fn get_cgroup_v2_memory_current(&self) -> io::Result<String>;
    fn get_cgroup_v2_memory_max(&self) -> io::Result<String>;
}
