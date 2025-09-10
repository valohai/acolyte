use crate::stats::cgroup_v1::CgroupV1Provider;
use std::io;
use tracing::debug;

/// Get total available memory from the cgroup v1 filesystem
pub fn get_memory_max_kb<P: CgroupV1Provider>(provider: &P) -> io::Result<u64> {
    // In cgroup v1, there are two ways to get the memory limit:
    // 1. `memory.limit_in_bytes` file
    // 2. `hierarchical_memory_limit` field in `memory.stat` field
    //
    // Hierarchical limit will be the more correct value to use as it also applies restrictions from parent cgroups.
    // In Kubernetes context, it will almost always be the same but there are a few corner cases where it might differ,
    // like pod post-scheduling changes to cgroups or misconfigurations.
    //
    // Using the `memory.stat->hierarchical_memory_limit` as the primary source of truth as it's the actual limit for OOM.

    match get_hierarchical_memory_limit(provider) {
        Ok(memory_limit) => {
            debug!("Using cgroup v1 for memory max (hierarchical_memory_limit)");
            return Ok(memory_limit / 1024);
        }
        Err(e) => {
            debug!("Failed to get hierarchical_memory_limit: {}", e);
        }
    }

    let memory_limit_text = provider.get_cgroup_v1_memory_limit_in_bytes()?;
    let memory_limit = memory_limit_text.trim().parse::<u64>().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid memory.limit_in_bytes format: {e}"),
        )
    })?;
    if memory_limit >= get_no_limit_value() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "`memory.limit_in_bytes` indicates no limit, cannot determine the actual memory limit",
        ));
    }

    let memory_limit_kb = memory_limit / 1024;
    debug!("Using cgroup v1 for memory max (limit_in_bytes)");
    Ok(memory_limit_kb)
}

fn get_hierarchical_memory_limit<P: CgroupV1Provider>(provider: &P) -> io::Result<u64> {
    let lines = provider.get_cgroup_v1_memory_stat()?;

    for line in lines {
        if line.starts_with("hierarchical_memory_limit ") {
            if let Some(value_str) = line.split_whitespace().nth(1) {
                if let Ok(value) = value_str.parse::<u64>() {
                    if value >= get_no_limit_value() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "hierarchical_memory_limit indicates no limit, cannot determine the actual memory limit",
                        ));
                    }

                    return Ok(value);
                }
            }

            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid hierarchical_memory_limit format in memory.stat",
            ));
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "hierarchical_memory_limit not found in memory.stat",
    ))
}

fn get_no_limit_value() -> u64 {
    // https://unix.stackexchange.com/questions/420906/what-is-the-value-for-the-cgroups-limit-in-bytes-if-the-memory-is-not-restricted
    // https://github.com/torvalds/linux/blob/76b6905c11fd3c6dc4562aefc3e8c4429fefae1e/include/linux/page_counter.h#L44-L48
    // In cgroup v1, the value "9223372036854771712" indicates "no limit" on 64-bit systems.
    // TODO: implement for 32-bit systems when needed
    9223372036854771712
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::cgroup_v1::MockCgroupV1Provider;

    #[test]
    fn test_prefers_hierarchical_memory_limit() -> io::Result<()> {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_memory_stat()
            .returning(|| {
                Ok(vec![
                    "cache 123456".to_string(),
                    "hierarchical_memory_limit 4194304".to_string(),
                    "total_swap 0".to_string(),
                ])
            });
        mock_provider
            .expect_get_cgroup_v1_memory_limit_in_bytes()
            .times(0);

        let memory_limit_kb = get_memory_max_kb(&mock_provider)?;
        assert_eq!(memory_limit_kb, 4096); // 4096 KB = 4194304 B
        Ok(())
    }

    #[test]
    fn test_unlimited_hierarchical_fallbacks() -> io::Result<()> {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_memory_stat()
            .returning(|| {
                Ok(vec![
                    "cache 123456".to_string(),
                    "hierarchical_memory_limit 9223372036854771712".to_string(),
                    "total_swap 0".to_string(),
                ])
            });
        mock_provider
            .expect_get_cgroup_v1_memory_limit_in_bytes()
            .returning(|| Ok("2097152\n".to_string()));

        let memory_limit_kb = get_memory_max_kb(&mock_provider)?;
        assert_eq!(memory_limit_kb, 2048); // 2048 KB = 2097152 B
        Ok(())
    }

    #[test]
    fn test_hierarchical_memory_limit_field_missing() -> io::Result<()> {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_memory_stat()
            .returning(|| Ok(vec!["cache 123456".to_string(), "total_swap 0".to_string()]));
        mock_provider
            .expect_get_cgroup_v1_memory_limit_in_bytes()
            .returning(|| Ok("2097152\n".to_string()));

        let memory_limit_kb = get_memory_max_kb(&mock_provider)?;
        assert_eq!(memory_limit_kb, 2048); // 2048 KB = 2097152 B
        Ok(())
    }

    #[test]
    fn test_memory_stat_access_fail_fallbacks() -> io::Result<()> {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_memory_stat()
            .returning(|| {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "memory.stat not found",
                ))
            });
        mock_provider
            .expect_get_cgroup_v1_memory_limit_in_bytes()
            .returning(|| Ok("2097152\n".to_string()));

        let memory_limit_kb = get_memory_max_kb(&mock_provider)?;
        assert_eq!(memory_limit_kb, 2048); // 2048 KB = 2097152 B
        Ok(())
    }

    #[test]
    fn test_secondary_source() -> io::Result<()> {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_memory_stat()
            .returning(|| Err(io::Error::new(io::ErrorKind::NotFound, "File not found")));
        mock_provider
            .expect_get_cgroup_v1_memory_limit_in_bytes()
            .returning(|| Ok("2097152\n".to_string()));

        let memory_limit_kb = get_memory_max_kb(&mock_provider)?;
        assert_eq!(memory_limit_kb, 2048); // 2048 KB = 2097152 B
        Ok(())
    }

    #[test]
    fn test_unlimited_memory_limit_as_fallback_is_error() {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_memory_stat()
            .returning(|| Err(io::Error::new(io::ErrorKind::NotFound, "File not found")));
        mock_provider
            .expect_get_cgroup_v1_memory_limit_in_bytes()
            .returning(|| Ok("9223372036854771712\n".to_string()));

        let result = get_memory_max_kb(&mock_provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_format() {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_memory_stat()
            .returning(|| Err(io::Error::new(io::ErrorKind::NotFound, "File not found")));
        mock_provider
            .expect_get_cgroup_v1_memory_limit_in_bytes()
            .returning(|| Ok("invalid\n".to_string()));

        let result = get_memory_max_kb(&mock_provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_double_io_errors() {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_memory_stat()
            .returning(|| Err(io::Error::new(io::ErrorKind::NotFound, "File not found")));
        mock_provider
            .expect_get_cgroup_v1_memory_limit_in_bytes()
            .returning(|| Err(io::Error::new(io::ErrorKind::NotFound, "File not found")));

        let result = get_memory_max_kb(&mock_provider);
        assert!(result.is_err());
    }
}
