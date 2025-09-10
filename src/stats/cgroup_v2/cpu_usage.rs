use crate::stats::CpuUsageValue;
use crate::stats::cgroup_v2::CgroupV2Provider;
use std::io;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Get normalized CPU usage from cgroup v2
pub fn get_cpu_usage<P: CgroupV2Provider>(
    provider: &P,
    sample_interval: Duration,
) -> io::Result<CpuUsageValue> {
    let start_time = Instant::now();

    let initial = get_cpu_usage_usec(provider)?;
    std::thread::sleep(sample_interval);
    let current = get_cpu_usage_usec(provider)?;

    // wall-clock time between the two readings
    let elapsed_usec = start_time.elapsed().as_micros() as f64;
    if elapsed_usec <= 0.0 {
        warn!("Elapsed time is zero or negative");
        return Ok(CpuUsageValue::FromCgroupV2(0.0));
    }

    // CPU time consumed between the two readings
    let delta_usage_usec = current.saturating_sub(initial) as f64;

    // Values from cgroup v2 are combined usage _time_ across all CPUs without idle times available,
    // so it's already the "normalized usage" we are familiar with:
    // - If a process used 100ms of CPU time in 100ms of real time, that is 1.0.
    // - If a process used 75ms of 2 CPUs in 100ms of real time, that is 1.5, but note that it's cumulative so cgroup reports 150ms
    let normalized_cpu_usage = delta_usage_usec / elapsed_usec;
    debug!("Using cgroup v2 for CPU usage");
    Ok(CpuUsageValue::FromCgroupV2(normalized_cpu_usage))
}

fn get_cpu_usage_usec<P: CgroupV2Provider>(provider: &P) -> io::Result<u64> {
    let lines = provider.get_cgroup_v2_cpu_stat()?;

    for line in lines {
        if line.starts_with("usage_usec") {
            if let Some(value_str) = line.split_whitespace().nth(1) {
                if let Ok(value) = value_str.parse::<u64>() {
                    return Ok(value);
                }
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Could not find usage_usec in v2 cgroup/cpu.stat",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::cgroup_v2::MockCgroupV2Provider;

    #[test]
    fn test_get_cpu_usage_usec() {
        let mut mock_provider = MockCgroupV2Provider::new();
        mock_provider.expect_get_cgroup_v2_cpu_stat().returning(|| {
            Ok(vec![
                "usage_usec 1000000".to_string(),
                "user_usec 800000".to_string(),
                "system_usec 200000".to_string(),
            ])
        });

        let microseconds = get_cpu_usage_usec(&mock_provider);
        assert!(microseconds.is_ok());
        assert_eq!(microseconds.unwrap(), 1000000);
    }
}
