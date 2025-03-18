use crate::stats::cgroup_v1::CgroupV1Provider;
use std::io;

/// Get the number of CPUs from the cgroup v1 filesystem
pub fn get_num_cpus<P: CgroupV1Provider>(_provider: &P) -> io::Result<f64> {
    todo!()
}
