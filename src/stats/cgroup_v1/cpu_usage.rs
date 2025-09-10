use crate::stats::CpuUsageValue;
use crate::stats::cgroup_v1::CgroupV1Provider;
use std::io;
use std::time::{Duration, Instant};
use tracing::debug;

/// Get normalized CPU usage from cgroup v1
pub fn get_cpu_usage<P: CgroupV1Provider>(
    provider: &P,
    sample_interval: Duration,
) -> io::Result<CpuUsageValue> {
    let start_time = Instant::now();

    // NB: cgroup v1 reports these cpu times in nanoseconds, unlike cgroup v2's microseconds
    let initial = get_cpu_usage_ns(provider)?;
    std::thread::sleep(sample_interval);
    let current = get_cpu_usage_ns(provider)?;

    // wall-clock time between the two readings
    let elapsed_ns = start_time.elapsed().as_nanos() as f64;
    if elapsed_ns <= 0.0 {
        return Err(io::Error::other(
            "Elapsed time between CPU measurements was zero or negative",
        ));
    }

    // CPU time consumed between the two readings
    let delta_usage_ns = current.saturating_sub(initial) as f64;

    let normalized_usage = delta_usage_ns / elapsed_ns;
    debug!("Using cgroup v1 for CPU usage");
    Ok(CpuUsageValue::FromCgroupV1(normalized_usage))
}

fn get_cpu_usage_ns<P: CgroupV1Provider>(provider: &P) -> io::Result<u64> {
    let cpuacct_usage_text = provider.get_cgroup_v1_cpuacct_usage()?;
    match cpuacct_usage_text.trim().parse::<u64>() {
        Ok(usage_ns) => Ok(usage_ns),
        Err(e) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid cpuacct.usage format: {e}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::cgroup_v1::MockCgroupV1Provider;

    #[test]
    fn test_get_cpu_usage_ns() {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_cpuacct_usage()
            .returning(|| Ok("12345678\n".to_string()));

        let usage_ns = get_cpu_usage_ns(&mock_provider).unwrap();
        assert_eq!(usage_ns, 12345678);
    }

    #[test]
    fn test_get_cpu_usage_ns_invalid_format() {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_cpuacct_usage()
            .returning(|| Ok("invalid\n".to_string()));

        let result = get_cpu_usage_ns(&mock_provider);
        assert!(result.is_err());
    }
}
