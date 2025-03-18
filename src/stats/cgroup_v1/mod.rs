use crate::stats::{CpuUsageValue, SystemStatsSource};
mod cpu_usage;
mod memory_current;
mod memory_max;
mod num_cpus;
use crate::utils::read_first_line;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

#[cfg(test)]
use mockall::automock;

#[derive(Default, Clone)]
pub struct CgroupV1MountPoints {
    pub cpu: Option<PathBuf>,
    pub cpuacct: Option<PathBuf>,
    pub memory: Option<PathBuf>,
}

pub struct CgroupV1Source<P: CgroupV1Provider> {
    provider: P,
}

impl<P: CgroupV1Provider> CgroupV1Source<P> {
    fn new(provider: P) -> Self {
        Self { provider }
    }
}

impl CgroupV1Source<CgroupV1FilesystemReader> {
    pub fn with_filesystem_reader_at(mount_points: CgroupV1MountPoints) -> Self {
        Self::new(CgroupV1FilesystemReader::new(mount_points))
    }
}

impl<P: CgroupV1Provider> SystemStatsSource for CgroupV1Source<P> {
    fn get_num_cpus(&self) -> io::Result<f64> {
        num_cpus::get_num_cpus(&self.provider)
    }

    fn get_cpu_usage(&self) -> io::Result<CpuUsageValue> {
        cpu_usage::get_cpu_usage(&self.provider)
    }

    fn get_memory_usage_kb(&self) -> io::Result<u64> {
        memory_current::get_memory_usage_kb(&self.provider)
    }

    fn get_memory_total_kb(&self) -> io::Result<u64> {
        memory_max::get_memory_max_kb(&self.provider)
    }
}

pub struct CgroupV1FilesystemReader {
    mount_points: CgroupV1MountPoints,
}

impl CgroupV1FilesystemReader {
    fn new(mount_points: CgroupV1MountPoints) -> Self {
        Self { mount_points }
    }

    fn cpu_quota_path(&self) -> Option<PathBuf> {
        self.mount_points
            .cpu
            .as_ref()
            .map(|pb| pb.join("cpu.cfs_quota_us"))
    }

    fn cpu_period_path(&self) -> Option<PathBuf> {
        self.mount_points
            .cpu
            .as_ref()
            .map(|pb| pb.join("cpu.cfs_period_us"))
    }

    fn cpu_usage(&self) -> Option<PathBuf> {
        self.mount_points
            .cpuacct
            .as_ref()
            .map(|pb| pb.join("cpuacct.usage"))
    }

    fn memory_usage_path(&self) -> Option<PathBuf> {
        self.mount_points
            .memory
            .as_ref()
            .map(|pb| pb.join("memory.usage_in_bytes"))
    }

    fn memory_limit_path(&self) -> Option<PathBuf> {
        self.mount_points
            .memory
            .as_ref()
            .map(|pb| pb.join("memory.limit_in_bytes"))
    }

    fn memory_stat_path(&self) -> Option<PathBuf> {
        self.mount_points
            .memory
            .as_ref()
            .map(|pb| pb.join("memory.stat"))
    }
}

#[cfg_attr(test, automock)]
pub trait CgroupV1Provider {
    fn get_cgroup_v1_cpu_cfs_quota(&self) -> io::Result<String>;
    fn get_cgroup_v1_cpu_cfs_period(&self) -> io::Result<String>;
    fn get_cgroup_v1_cpuacct_usage(&self) -> io::Result<String>;
    fn get_cgroup_v1_memory_usage_in_bytes(&self) -> io::Result<String>;
    fn get_cgroup_v1_memory_limit_in_bytes(&self) -> io::Result<String>;
    fn get_cgroup_v1_memory_stat(&self) -> io::Result<Vec<String>>;
}

impl CgroupV1Provider for CgroupV1FilesystemReader {
    fn get_cgroup_v1_cpu_cfs_quota(&self) -> io::Result<String> {
        let Some(file_path) = self.cpu_quota_path() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "cpu.cfs_quota_us file not found",
            ));
        };

        read_first_line(file_path)
    }

    fn get_cgroup_v1_cpu_cfs_period(&self) -> io::Result<String> {
        let Some(file_path) = self.cpu_period_path() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "cpu.cfs_period_us file not found",
            ));
        };

        read_first_line(file_path)
    }

    fn get_cgroup_v1_cpuacct_usage(&self) -> io::Result<String> {
        let Some(file_path) = self.cpu_usage() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "cpuacct.usage file not found",
            ));
        };

        read_first_line(file_path)
    }

    fn get_cgroup_v1_memory_usage_in_bytes(&self) -> io::Result<String> {
        let Some(file_path) = self.memory_usage_path() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "memory.usage_in_bytes file not found",
            ));
        };

        read_first_line(file_path)
    }

    fn get_cgroup_v1_memory_limit_in_bytes(&self) -> io::Result<String> {
        let Some(file_path) = self.memory_limit_path() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "memory.limit_in_bytes file not found",
            ));
        };

        read_first_line(file_path)
    }

    fn get_cgroup_v1_memory_stat(&self) -> io::Result<Vec<String>> {
        let Some(file_path) = self.memory_stat_path() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "memory.stat file not found",
            ));
        };

        let file = File::open(file_path)?;
        BufReader::new(file).lines().collect()
    }
}
