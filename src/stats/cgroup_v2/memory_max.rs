use crate::stats::cgroup_v2::CgroupV2Provider;
use std::io;

/// Get total available memory from the cgroup v2 filesystem
pub fn get_memory_max_kb<P: CgroupV2Provider>(provider: &P) -> io::Result<u64> {
    let memory_max_text = provider.get_cgroup_v2_memory_max()?;

    if memory_max_text.trim() == "max" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "`memory.max` contains 'max' (unlimited), cannot determine the actual memory limit",
        ));
    }

    match memory_max_text.trim().parse::<u64>() {
        Ok(mem_bytes) => Ok(mem_bytes / 1024),
        Err(e) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid memory.max format: {}", e),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::cgroup_v2::MockCgroupV2Provider;

    #[test]
    fn test_get_memory_max_kb_normal() -> io::Result<()> {
        let mut mock_provider = MockCgroupV2Provider::new();
        mock_provider
            .expect_get_cgroup_v2_memory_max()
            .returning(|| Ok("1048576".to_string())); // 1MB in B

        let memory_max_kb = get_memory_max_kb(&mock_provider)?;
        assert_eq!(memory_max_kb, 1024); // 1MB in KB
        Ok(())
    }

    #[test]
    fn test_get_memory_max_kb_unlimited() {
        let mut mock_provider = MockCgroupV2Provider::new();
        mock_provider
            .expect_get_cgroup_v2_memory_max()
            .returning(|| Ok("max".to_string()));

        let result = get_memory_max_kb(&mock_provider);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unlimited"));
    }

    #[test]
    fn test_get_memory_max_kb_invalid_format() {
        let mut mock_provider = MockCgroupV2Provider::new();
        mock_provider
            .expect_get_cgroup_v2_memory_max()
            .returning(|| Ok("invalid".to_string()));

        let result = get_memory_max_kb(&mock_provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_memory_max_kb_io_error() {
        let mut mock_provider = MockCgroupV2Provider::new();
        mock_provider
            .expect_get_cgroup_v2_memory_max()
            .returning(|| Err(io::Error::new(io::ErrorKind::NotFound, "File not found")));

        let result = get_memory_max_kb(&mock_provider);
        assert!(result.is_err());
    }
}
