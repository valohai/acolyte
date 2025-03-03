use crate::stats::cgroup_v2::CgroupV2Provider;
use std::io;

/// Get currently used memory from the cgroup v2 filesystem
pub fn get_memory_current_kb<P: CgroupV2Provider>(provider: &P) -> io::Result<u64> {
    let memory_current_text = provider.get_cgroup_v2_memory_current()?;

    match memory_current_text.trim().parse::<u64>() {
        Ok(mem_bytes) => Ok(mem_bytes / 1024),
        Err(e) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid memory.current format: {}", e),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::cgroup_v2::MockCgroupV2Provider;

    #[test]
    fn test_get_memory_current_kb_normal() -> io::Result<()> {
        let mut mock_provider = MockCgroupV2Provider::new();
        mock_provider
            .expect_get_cgroup_v2_memory_current()
            .returning(|| Ok("1048576".to_string())); // 1MB in B

        let memory_current_kb = get_memory_current_kb(&mock_provider)?;
        assert_eq!(memory_current_kb, 1024); // 1MB in KB
        Ok(())
    }

    #[test]
    fn test_get_memory_current_kb_invalid_format() {
        let mut mock_provider = MockCgroupV2Provider::new();
        mock_provider
            .expect_get_cgroup_v2_memory_current()
            .returning(|| Ok("invalid".to_string()));

        let result = get_memory_current_kb(&mock_provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_memory_current_kb_io_error() {
        let mut mock_provider = MockCgroupV2Provider::new();
        mock_provider
            .expect_get_cgroup_v2_memory_current()
            .returning(|| Err(io::Error::new(io::ErrorKind::NotFound, "File not found")));

        let result = get_memory_current_kb(&mock_provider);
        assert!(result.is_err());
    }
}
