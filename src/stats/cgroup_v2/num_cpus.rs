use crate::stats::cgroup_v2::CgroupV2Provider;
use std::io;

/// Get the number of CPUs from the cgroup v2 filesystem
pub fn get_num_cpus<P: CgroupV2Provider>(provider: &P) -> io::Result<f64> {
    let cpu_max_text = provider.get_cgroup_v2_cpu_max()?;

    // In cgroup v2, CPU quotas are specified in the `cpu.max` file with two numbers:
    // - quota: maximum amount of CPU time (in microseconds) that the cgroup can use per period
    // - period: length of the scheduling period (in microseconds)
    //
    // So, the number of CPUs "cores" = quota / period

    // `cpu.max` format: "quota period"
    let parts: Vec<&str> = cpu_max_text.trim().split_whitespace().collect();
    if parts.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid cpu.max format: {}", cpu_max_text),
        ));
    }

    let quota_str = parts[0];
    let period_str = parts[1];

    if quota_str == "max" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "`cpu.max` contains 'max' quota (unlimited), cannot determine the actual CPU count",
        ));
    }

    let quota = match quota_str.parse::<u64>() {
        Ok(q) => q,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse CPU quota '{}': {}", quota_str, e),
            ));
        }
    };

    let period = match period_str.parse::<u64>() {
        Ok(p) => p,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse CPU period '{}': {}", period_str, e),
            ));
        }
    };

    let num_cpus = quota as f64 / period as f64;
    Ok(num_cpus)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::cgroup_v2::MockCgroupV2Provider;

    #[test]
    fn test_get_num_cpus_with_quota() {
        let mut mock_provider = MockCgroupV2Provider::new();

        mock_provider
            .expect_get_cgroup_v2_cpu_max()
            .returning(|| Ok("200000 100000".to_string()));

        assert_eq!(get_num_cpus(&mock_provider).unwrap(), 2.0);
    }

    #[test]
    fn test_get_num_cpus_with_fractional_quota() {
        let mut mock_provider = MockCgroupV2Provider::new();

        mock_provider
            .expect_get_cgroup_v2_cpu_max()
            .returning(|| Ok("50000 100000".to_string()));

        assert_eq!(get_num_cpus(&mock_provider).unwrap(), 0.5);
    }

    #[test]
    fn test_get_num_cpus_with_no_quota() {
        let mut mock_provider = MockCgroupV2Provider::new();

        mock_provider
            .expect_get_cgroup_v2_cpu_max()
            .returning(|| Ok("max 100000".to_string()));

        let result = get_num_cpus(&mock_provider);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unlimited"));
    }

    #[test]
    fn test_get_num_cpus_with_invalid_format() {
        let mut mock_provider = MockCgroupV2Provider::new();

        mock_provider
            .expect_get_cgroup_v2_cpu_max()
            .returning(|| Ok("200000".to_string()));

        assert!(get_num_cpus(&mock_provider).is_err());
    }

    #[test]
    fn test_get_num_cpus_with_invalid_quota() {
        let mut mock_provider = MockCgroupV2Provider::new();

        mock_provider
            .expect_get_cgroup_v2_cpu_max()
            .returning(|| Ok("invalid 100000".to_string()));

        assert!(get_num_cpus(&mock_provider).is_err());
    }

    #[test]
    fn test_get_num_cpus_with_invalid_period() {
        let mut mock_provider = MockCgroupV2Provider::new();

        mock_provider
            .expect_get_cgroup_v2_cpu_max()
            .returning(|| Ok("200000 invalid".to_string()));

        assert!(get_num_cpus(&mock_provider).is_err());
    }
}
