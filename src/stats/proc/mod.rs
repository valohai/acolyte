mod cpu_usage;
mod memory;
mod num_cpus;

use crate::stats::{utils, ResourceType, SystemStatsSource};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use tracing::debug;

#[cfg(test)]
use mockall::automock;

/// A source of system stats that reads values like `/proc` provides.
pub struct ProcSource<P: ProcProvider> {
    provider: P,
}

impl<P: ProcProvider> ProcSource<P> {
    fn new(provider: P) -> Self {
        Self { provider }
    }
}

impl<P: ProcProvider> SystemStatsSource for ProcSource<P> {
    fn get_num_cpus(&self) -> io::Result<f64> {
        debug!("Using /proc for the number of CPUs");
        num_cpus::get_num_cpus(&self.provider)
    }

    fn get_cpu_usage(&self) -> io::Result<f64> {
        debug!("Using /proc for CPU usage");
        cpu_usage::get_cpu_usage(&self.provider)
    }

    fn get_memory_usage_kb(&self) -> io::Result<u64> {
        debug!("Using /proc for memory usage kb");
        let (memory_usage_kb, _) = memory::get_memory_usage_and_total_kb(&self.provider)?;
        Ok(memory_usage_kb)
    }

    fn get_memory_total_kb(&self) -> io::Result<u64> {
        debug!("Using /proc for memory total kb");
        let (_, memory_total_kb) = memory::get_memory_usage_and_total_kb(&self.provider)?;
        Ok(memory_total_kb)
    }

    fn is_available_for(&self, resource_type: &ResourceType) -> bool {
        self.provider.is_available_for(resource_type)
    }
}

impl ProcSource<ProcFilesystemReader> {
    pub fn with_filesystem_reader_at(path: &str) -> Self {
        Self::new(ProcFilesystemReader::new(path))
    }
}

/// The proc value provider that reads from a target filesystem like `/proc`.
pub struct ProcFilesystemReader {
    proc_path: PathBuf,
}

impl ProcFilesystemReader {
    fn new(path: &str) -> Self {
        Self {
            proc_path: PathBuf::from(path),
        }
    }

    fn proc_stat_path(&self) -> PathBuf {
        self.proc_path.join("stat")
    }

    fn proc_meminfo_path(&self) -> PathBuf {
        self.proc_path.join("meminfo")
    }
}

impl ProcProvider for ProcFilesystemReader {
    fn get_proc_stat(&self) -> io::Result<Vec<String>> {
        let file = File::open(self.proc_stat_path())?;
        BufReader::new(file).lines().collect()
    }

    fn get_proc_meminfo(&self) -> io::Result<Vec<String>> {
        let file = File::open(self.proc_meminfo_path())?;
        BufReader::new(file).lines().collect()
    }

    fn is_available_for(&self, resource_type: &ResourceType) -> bool {
        let path_to_check = match resource_type {
            ResourceType::NumCpus | ResourceType::CpuUsage => self.proc_stat_path(),
            ResourceType::MemoryUsageKb | ResourceType::MemoryTotalKb => self.proc_meminfo_path(),
        };
        utils::is_file_readable(&path_to_check)
    }
}

/// The implementer provides proc values from somewhere, useful for mocking in tests
#[cfg_attr(test, automock)]
pub trait ProcProvider {
    fn get_proc_stat(&self) -> io::Result<Vec<String>>;
    fn get_proc_meminfo(&self) -> io::Result<Vec<String>>;
    fn is_available_for(&self, resource_type: &ResourceType) -> bool;
}
