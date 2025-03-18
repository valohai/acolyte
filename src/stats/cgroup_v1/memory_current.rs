use crate::stats::cgroup_v1::CgroupV1Provider;
use std::io;

/// Get currently used memory from the cgroup v1 filesystem
pub fn get_memory_usage_kb<P: CgroupV1Provider>(_provider: &P) -> io::Result<u64> {
    todo!()
}
