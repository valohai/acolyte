use crate::stats::cgroup_v1::CgroupV1Provider;
use crate::stats::CpuUsageValue;
use std::io;

/// Get normalized CPU usage from cgroup v1
pub fn get_cpu_usage<P: CgroupV1Provider>(_provider: &P) -> io::Result<CpuUsageValue> {
    todo!()
}
