mod cpu_usage;
mod memory;
mod num_cpus;

use crate::stats::{CpuUsageValue, SystemStatsSource};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

#[cfg(test)]
use mockall::automock;
use tracing::debug;

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
        num_cpus::get_num_cpus(&self.provider)
    }

    fn get_cpu_usage(&self) -> io::Result<CpuUsageValue> {
        cpu_usage::get_cpu_usage(&self.provider)
    }

    fn get_memory_usage_kb(&self) -> io::Result<u64> {
        let (memory_usage_kb, _) = memory::get_memory_usage_and_total_kb(&self.provider)?;
        debug!("Using proc for memory usage");
        Ok(memory_usage_kb)
    }

    fn get_memory_total_kb(&self) -> io::Result<u64> {
        let (_, memory_total_kb) = memory::get_memory_usage_and_total_kb(&self.provider)?;
        debug!("Using proc for memory max");
        Ok(memory_total_kb)
    }
}

impl ProcSource<ProcFilesystemReader> {
    pub fn with_filesystem_reader_at(path: PathBuf) -> Self {
        Self::new(ProcFilesystemReader::new(path))
    }
}

/// The proc value provider that reads from a target filesystem like `/proc`.
pub struct ProcFilesystemReader {
    proc_path: PathBuf,
}

impl ProcFilesystemReader {
    fn new(proc_path: PathBuf) -> Self {
        Self { proc_path }
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
}

/// The implementer provides proc values from somewhere, useful for mocking in tests
#[cfg_attr(test, automock)]
pub trait ProcProvider {
    fn get_proc_stat(&self) -> io::Result<Vec<String>>;
    fn get_proc_meminfo(&self) -> io::Result<Vec<String>>;
}
