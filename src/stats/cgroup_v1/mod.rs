use crate::stats::{CpuUsageValue, ResourceType, SystemStatsSource};
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

#[derive(Default)]
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
        debug!("Using cgroup v1 for the number of CPUs");
        num_cpus::get_num_cpus(&self.provider)
    }

    fn get_cpu_usage(&self) -> io::Result<CpuUsageValue> {
        debug!("Using cgroup v1 for CPU usage");
        cpu_usage::get_cpu_usage(&self.provider)
    }

    fn get_memory_usage_kb(&self) -> io::Result<u64> {
        debug!("Using cgroup v1 for memory usage");
        memory_current::get_memory_usage_kb(&self.provider)
    }

    fn get_memory_total_kb(&self) -> io::Result<u64> {
        debug!("Using cgroup v1 for memory total");
        memory_max::get_memory_max_kb(&self.provider)
    }

    fn is_available_for(&self, resource_type: &ResourceType) -> bool {
        self.provider.is_available_for(resource_type)
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
    fn is_available_for(&self, resource_type: &ResourceType) -> bool;
}

impl CgroupV1Provider for CgroupV1FilesystemReader {
    fn get_cgroup_v1_cpu_cfs_quota(&self) -> io::Result<String> {
        let Some(file_path) = self.cpu_quota_path() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "cpu.cfs_quota_us file not found",
            ));
        };

        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line)
    }

    fn get_cgroup_v1_cpu_cfs_period(&self) -> io::Result<String> {
        let Some(file_path) = self.cpu_period_path() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "cpu.cfs_period_us file not found",
            ));
        };

        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line)
    }

    fn get_cgroup_v1_cpuacct_usage(&self) -> io::Result<String> {
        let Some(file_path) = self.cpu_usage() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "cpuacct.usage file not found",
            ));
        };

        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line)
    }

    fn get_cgroup_v1_memory_usage_in_bytes(&self) -> io::Result<String> {
        let Some(file_path) = self.memory_usage_path() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "memory.usage_in_bytes file not found",
            ));
        };

        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line)
    }

    fn get_cgroup_v1_memory_limit_in_bytes(&self) -> io::Result<String> {
        let Some(file_path) = self.memory_limit_path() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "memory.limit_in_bytes file not found",
            ));
        };

        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line)
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

    fn is_available_for(&self, resource_type: &ResourceType) -> bool {
        match resource_type {
            ResourceType::NumCpus => {
                self.get_cgroup_v1_cpu_cfs_quota().is_ok()
                    && self.get_cgroup_v1_cpu_cfs_period().is_ok()
            }
            ResourceType::CpuUsage => self.get_cgroup_v1_cpuacct_usage().is_ok(),
            ResourceType::MemoryUsageKb => self.get_cgroup_v1_memory_usage_in_bytes().is_ok(),
            ResourceType::MemoryTotalKb => {
                // TODO: this does not take into account that these can be "no limit"
                self.get_cgroup_v1_memory_limit_in_bytes().is_ok()
                    || self.get_cgroup_v1_memory_stat().is_ok()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_all_available() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();

        let cpu_path = temp_path.join("cpu,cpuacct");
        let cpuacct_path = cpu_path.clone();
        let memory_path = temp_path.join("memory");
        std::fs::create_dir_all(&cpu_path)?;
        std::fs::create_dir_all(&memory_path)?;

        let cpu_quota_path = cpu_path.join("cpu.cfs_quota_us");
        let mut file = File::create(&cpu_quota_path)?;
        file.write_all(b"100001\n")?;

        let cpu_period_path = cpu_path.join("cpu.cfs_period_us");
        let mut file = File::create(&cpu_period_path)?;
        file.write_all(b"100002\n")?;

        let cpuacct_usage_path = cpuacct_path.join("cpuacct.usage");
        let mut file = File::create(&cpuacct_usage_path)?;
        file.write_all(b"100003\n")?;

        let mem_usage_path = memory_path.join("memory.usage_in_bytes");
        let mut file = File::create(&mem_usage_path)?;
        file.write_all(b"100004\n")?;

        let mem_limit_path = memory_path.join("memory.limit_in_bytes");
        let mut file = File::create(&mem_limit_path)?;
        file.write_all(b"100005\n")?;

        let mount_points = CgroupV1MountPoints {
            cpu: Some(cpu_path),
            cpuacct: Some(cpuacct_path),
            memory: Some(memory_path),
        };
        let reader = CgroupV1FilesystemReader::new(mount_points);
        assert!(reader.is_available_for(&ResourceType::NumCpus));
        assert!(reader.is_available_for(&ResourceType::CpuUsage));
        assert!(reader.is_available_for(&ResourceType::MemoryUsageKb));
        assert!(reader.is_available_for(&ResourceType::MemoryTotalKb));

        Ok(())
    }

    #[test]
    fn test_memory_max_can_be_read_from_memory_stat_too() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();

        let memory_path = temp_path.join("memory");
        std::fs::create_dir_all(&memory_path)?;

        let mem_stats_path = memory_path.join("memory.stat");
        let mut file = File::create(&mem_stats_path)?;
        file.write_all(b"abc 123\ndef 456")?;

        let mount_points = CgroupV1MountPoints {
            cpu: None,
            cpuacct: None,
            memory: Some(memory_path),
        };
        let reader = CgroupV1FilesystemReader::new(mount_points);
        assert!(reader.is_available_for(&ResourceType::MemoryTotalKb));

        Ok(())
    }

    #[test]
    fn test_nothing_available() {
        let mount_points = CgroupV1MountPoints::default();
        let reader = CgroupV1FilesystemReader::new(mount_points);
        assert!(!reader.is_available_for(&ResourceType::NumCpus));
        assert!(!reader.is_available_for(&ResourceType::CpuUsage));
        assert!(!reader.is_available_for(&ResourceType::MemoryUsageKb));
        assert!(!reader.is_available_for(&ResourceType::MemoryTotalKb));
    }
}
