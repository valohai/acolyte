use crate::stats::cgroup_v1::CgroupV1Provider;
use std::io;
use tracing::debug;

/// Get currently used memory from the cgroup v1 filesystem
pub fn get_memory_usage_kb<P: CgroupV1Provider>(provider: &P) -> io::Result<u64> {
    let memory_usage_text = provider.get_cgroup_v1_memory_usage_in_bytes()?;

    match memory_usage_text.trim().parse::<u64>() {
        Ok(mem_bytes) => {
            debug!("Using cgroup v1 for memory usage");
            Ok(mem_bytes / 1024)
        }
        Err(e) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid memory.usage_in_bytes format: {e}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::cgroup_v1::MockCgroupV1Provider;

    #[test]
    fn test_get_memory_usage_kb_normal() -> io::Result<()> {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_memory_usage_in_bytes()
            .returning(|| Ok("1048576\n".to_string())); // 1MB in bytes

        let memory_usage_kb = get_memory_usage_kb(&mock_provider)?;
        assert_eq!(memory_usage_kb, 1024); // 1MB in KB
        Ok(())
    }

    #[test]
    fn test_get_memory_usage_kb_invalid_format() {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_memory_usage_in_bytes()
            .returning(|| Ok("invalid\n".to_string()));

        let result = get_memory_usage_kb(&mock_provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_memory_usage_kb_io_error() {
        let mut mock_provider = MockCgroupV1Provider::new();
        mock_provider
            .expect_get_cgroup_v1_memory_usage_in_bytes()
            .returning(|| Err(io::Error::new(io::ErrorKind::NotFound, "File not found")));

        let result = get_memory_usage_kb(&mock_provider);
        assert!(result.is_err());
    }
}
