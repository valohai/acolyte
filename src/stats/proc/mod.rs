mod cpu;
mod mem;

use crate::stats::{utils, CpuStats, MemoryStats, ResourceType, SystemStatsSource};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use tracing::debug;

#[cfg(test)]
use mockall::automock;

/// A source of system stats that reads from the `/proc` filesystem (by default).
pub struct ProcSource<P: ProcProvider> {
    provider: P,
}

impl<P: ProcProvider> ProcSource<P> {
    fn new(provider: P) -> Self {
        Self { provider }
    }
}

impl<P: ProcProvider> SystemStatsSource for ProcSource<P> {
    fn get_cpu_stats(&self) -> io::Result<CpuStats> {
        debug!("Using /proc for CPU stats");
        cpu::get_cpu_stats(&self.provider)
    }

    fn get_memory_stats(&self) -> io::Result<MemoryStats> {
        debug!("Using /proc for memory stats");
        mem::get_memory_stats(&self.provider)
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

/// The default proc value provider, reads from the `/proc` filesystem.
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
        let patch_to_check = match resource_type {
            ResourceType::CPU => self.proc_stat_path(),
            ResourceType::Memory => self.proc_meminfo_path(),
        };
        utils::is_file_readable(&patch_to_check)
    }
}

/// The implementer provides proc values from somewhere, useful for mocking in tests
#[cfg_attr(test, automock)]
pub trait ProcProvider {
    fn get_proc_stat(&self) -> io::Result<Vec<String>>;
    fn get_proc_meminfo(&self) -> io::Result<Vec<String>>;
    fn is_available_for(&self, resource_type: &ResourceType) -> bool;
}
