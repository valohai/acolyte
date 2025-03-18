use crate::stats::cgroup_v1::CgroupV1Provider;
use std::io;

/// Get total available memory from the cgroup v1 filesystem
pub fn get_memory_max_kb<P: CgroupV1Provider>(_provider: &P) -> io::Result<u64> {
    todo!()
}

