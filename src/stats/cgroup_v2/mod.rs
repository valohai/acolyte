use crate::stats::{CpuUsageValue, ResourceType, SystemStatsSource, utils};
mod cpu_usage;
mod memory_current;
mod memory_max;
mod num_cpus;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use tracing::debug;

#[cfg(test)]
use mockall::automock;

pub struct CgroupV2Source<P: CgroupV2Provider> {
    provider: P,
}

impl<P: CgroupV2Provider> CgroupV2Source<P> {
    fn new(provider: P) -> Self {
        Self { provider }
    }
}

impl CgroupV2Source<CgroupV2FilesystemReader> {
    pub fn with_filesystem_reader_at(path: &str) -> Self {
        Self::new(CgroupV2FilesystemReader::new(path))
    }
}

impl<P: CgroupV2Provider> SystemStatsSource for CgroupV2Source<P> {
    fn get_num_cpus(&self) -> io::Result<f64> {
        debug!("Using cgroup v2 for the number of CPUs");
        num_cpus::get_num_cpus(&self.provider)
    }

    fn get_cpu_usage(&self) -> io::Result<CpuUsageValue> {
        debug!("Using cgroup v2 for CPU usage");
        cpu_usage::get_cpu_usage(&self.provider)
    }

    fn get_memory_usage_kb(&self) -> io::Result<u64> {
        debug!("Using cgroup v2 for memory usage");
        memory_current::get_memory_current_kb(&self.provider)
    }

    fn get_memory_total_kb(&self) -> io::Result<u64> {
        debug!("Using cgroup v2 for memory total");
        memory_max::get_memory_max_kb(&self.provider)
    }

    fn is_available_for(&self, resource_type: &ResourceType) -> bool {
        self.provider.is_available_for(resource_type)
    }
}

pub struct CgroupV2FilesystemReader {
    cgroup_v2_path: PathBuf,
}

impl CgroupV2FilesystemReader {
    fn new(path: &str) -> Self {
        Self {
            cgroup_v2_path: PathBuf::from(path),
        }
    }

    fn cpu_max_path(&self) -> PathBuf {
        self.cgroup_v2_path.join("cpu.max")
    }

    fn cpu_stat_path(&self) -> PathBuf {
        self.cgroup_v2_path.join("cpu.stat")
    }

    fn mem_current_path(&self) -> PathBuf {
        self.cgroup_v2_path.join("memory.current")
    }

    fn mem_max_path(&self) -> PathBuf {
        self.cgroup_v2_path.join("memory.max")
    }
}

impl CgroupV2Provider for CgroupV2FilesystemReader {
    fn get_cgroup_v2_cpu_stat(&self) -> io::Result<Vec<String>> {
        let file = File::open(self.cpu_stat_path())?;
        BufReader::new(file).lines().collect()
    }

    fn get_cgroup_v2_cpu_max(&self) -> io::Result<String> {
        let file = File::open(self.cpu_max_path())?;
        let mut reader = BufReader::new(file);
        // `cpu.max` file is just a single line...
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line)
    }

    fn get_cgroup_v2_memory_current(&self) -> io::Result<String> {
        let file = File::open(self.mem_current_path())?;
        let mut reader = BufReader::new(file);
        // `memory.current` file is just a single line...
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line)
    }

    fn get_cgroup_v2_memory_max(&self) -> io::Result<String> {
        let file = File::open(self.mem_max_path())?;
        let mut reader = BufReader::new(file);
        // `memory.max` file is just a single line...
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line)
    }

    fn is_available_for(&self, resource_type: &ResourceType) -> bool {
        let path_to_check = match resource_type {
            ResourceType::NumCpus => self.cpu_max_path(),
            ResourceType::CpuUsage => self.cpu_stat_path(),
            ResourceType::MemoryUsageKb => self.mem_current_path(),
            ResourceType::MemoryTotalKb => self.mem_max_path(),
        };

        if !utils::is_file_readable(&path_to_check) {
            return false;
        }

        // additional checks where just checking that the file is readable is not enough
        match resource_type {
            ResourceType::MemoryTotalKb => match File::open(&path_to_check) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    let mut first_line = String::new();
                    if reader.read_line(&mut first_line).is_ok() && first_line.trim() == "max" {
                        debug!(
                            "`memory.max` contains 'max', source unavailable for getting the actual memory total"
                        );
                        return false;
                    }
                }
                Err(_) => return false,
            },
            ResourceType::NumCpus => match File::open(&path_to_check) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    let mut first_line = String::new();
                    if reader.read_line(&mut first_line).is_ok() {
                        let parts: Vec<&str> = first_line.split_whitespace().collect();
                        if !parts.is_empty() && parts[0] == "max" {
                            debug!(
                                "`cpu.max` contains 'max' quota, source unavailable for getting the actual number of CPUs"
                            );
                            return false;
                        }
                    }
                }
                Err(_) => return false,
            },
            _ => {}
        }

        true
    }
}

#[cfg_attr(test, automock)]
pub trait CgroupV2Provider {
    fn get_cgroup_v2_cpu_stat(&self) -> io::Result<Vec<String>>;
    fn get_cgroup_v2_cpu_max(&self) -> io::Result<String>;
    fn get_cgroup_v2_memory_current(&self) -> io::Result<String>;
    fn get_cgroup_v2_memory_max(&self) -> io::Result<String>;
    fn is_available_for(&self, resource_type: &ResourceType) -> bool;
}
