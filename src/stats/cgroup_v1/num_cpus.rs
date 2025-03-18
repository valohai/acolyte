use crate::stats::cgroup_v1::CgroupV1Provider;
use std::io;

/// Get the number of CPUs from the cgroup v1 filesystem
pub fn get_num_cpus<P: CgroupV1Provider>(provider: &P) -> io::Result<f64> {
    let quota_text = provider.get_cgroup_v1_cpu_cfs_quota()?;
    let period_text = provider.get_cgroup_v1_cpu_cfs_period()?;

    // In cgroup v1, CPU limit / count can be resolved from:
    // - cpu.cfs_quota_us: maximum time (in microseconds) the cgroup can run per period
    // - cpu.cfs_period_us: period length (in microseconds)
    //
    // so, the number of CPUs "cores" = quota / period

    let quota: i64 = quota_text.trim().parse().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid cpu.cfs_quota_us format: {}", e),
        )
    })?;
    if quota <= 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cpu.cfs_quota_us is zero or less (unlimited), cannot determine the actual CPU count",
        ));
    }

    let period: u64 = period_text.trim().parse().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid cpu.cfs_period_us format: {}", e),
        )
    })?;
    if period == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cpu.cfs_period_us is zero or less, cannot determine the actual CPU count",
        ));
    }

    let num_cpus = quota as f64 / period as f64;
    Ok(num_cpus)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::cgroup_v1::MockCgroupV1Provider;

    #[test]
    fn test_normal() -> io::Result<()> {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_quota()
            .returning(|| Ok("200000\n".to_string()));
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_period()
            .returning(|| Ok("100000\n".to_string()));

        let num_cpus = get_num_cpus(&mock_provider)?;
        assert_eq!(num_cpus, 2.0);
        Ok(())
    }

    #[test]
    fn test_fractional() -> io::Result<()> {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_quota()
            .returning(|| Ok("50000\n".to_string()));
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_period()
            .returning(|| Ok("100000\n".to_string()));

        let num_cpus = get_num_cpus(&mock_provider)?;
        assert_eq!(num_cpus, 0.5);
        Ok(())
    }

    #[test]
    fn test_unlimited_quota() {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_quota()
            .returning(|| Ok("-1\n".to_string()));
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_period()
            .returning(|| Ok("100000\n".to_string()));

        let result = get_num_cpus(&mock_provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_period() {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_quota()
            .returning(|| Ok("100000\n".to_string()));
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_period()
            .returning(|| Ok("0\n".to_string()));

        let result = get_num_cpus(&mock_provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_quota() {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_quota()
            .returning(|| Ok("invalid\n".to_string()));
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_period()
            .returning(|| Ok("100000\n".to_string()));

        let result = get_num_cpus(&mock_provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_period() {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_quota()
            .returning(|| Ok("100000\n".to_string()));
        mock_provider
            .expect_get_cgroup_v1_cpu_cfs_period()
            .returning(|| Ok("invalid\n".to_string()));

        let result = get_num_cpus(&mock_provider);
        assert!(result.is_err());
    }
}
