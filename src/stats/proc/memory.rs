use crate::stats::proc::ProcProvider;
use std::io;

/// Get currently used and total available memory from the `/proc` filesystem (host-wide)
pub fn get_memory_usage_and_total_kb<R: ProcProvider>(provider: &R) -> io::Result<(u64, u64)> {
    let lines = provider.get_proc_meminfo()?;

    let mut memory_total_kb = 0;
    let mut available_kb = 0;

    for line in &lines {
        if line.starts_with("MemAvailable:") {
            available_kb = parse_proc_meminfo_value(line);
        } else if line.starts_with("MemTotal:") {
            memory_total_kb = parse_proc_meminfo_value(line);
        }

        if memory_total_kb > 0 && available_kb > 0 {
            break;
        }
    }

    let memory_usage_kb = memory_total_kb.saturating_sub(available_kb);
    Ok((memory_usage_kb, memory_total_kb))
}

fn parse_proc_meminfo_value(line: &str) -> u64 {
    // most /proc/meminfo lines look like this:
    // MemTotal: 8048836 kB
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::proc::MockProcProvider;

    #[test]
    fn test_get_memory_stats_normal() -> io::Result<()> {
        let mut mock_provider = MockProcProvider::new();
        mock_provider.expect_get_proc_meminfo().returning(|| {
            Ok(vec![
                "MemTotal:        8048836 kB".to_string(),
                "MemFree:         2000000 kB".to_string(),
                "MemAvailable:    4019418 kB".to_string(),
            ])
        });

        let (memory_usage_kb, memory_total_kb) = get_memory_usage_and_total_kb(&mock_provider)?;
        assert_eq!(memory_total_kb, 8048836);
        assert_eq!(memory_usage_kb, 4029418);
        Ok(())
    }

    #[test]
    fn test_get_memory_stats_missing_available() -> io::Result<()> {
        let mut mock_provider = MockProcProvider::new();
        mock_provider.expect_get_proc_meminfo().returning(|| {
            Ok(vec![
                "MemTotal:        8048836 kB".to_string(),
                "MemFree:         2000000 kB".to_string(),
                // Missing MemAvailable
            ])
        });

        let (memory_usage_kb, memory_total_kb) = get_memory_usage_and_total_kb(&mock_provider)?;
        assert_eq!(memory_total_kb, 8048836);
        assert_eq!(memory_usage_kb, 8048836);
        Ok(())
    }

    #[test]
    fn test_get_memory_stats_missing_total() -> io::Result<()> {
        let mut mock_provider = MockProcProvider::new();
        mock_provider.expect_get_proc_meminfo().returning(|| {
            Ok(vec![
                // Missing MemTotal
                "MemFree:         2000000 kB".to_string(),
                "MemAvailable:    4019418 kB".to_string(),
            ])
        });

        let (memory_usage_kb, memory_total_kb) = get_memory_usage_and_total_kb(&mock_provider)?;
        assert_eq!(memory_total_kb, 0);
        assert_eq!(memory_usage_kb, 0);
        Ok(())
    }

    #[test]
    fn test_get_memory_stats_empty_meminfo() -> io::Result<()> {
        let mut mock_provider = MockProcProvider::new();
        mock_provider
            .expect_get_proc_meminfo()
            .returning(|| Ok(vec![]));

        let (memory_usage_kb, memory_total_kb) = get_memory_usage_and_total_kb(&mock_provider)?;
        assert_eq!(memory_total_kb, 0);
        assert_eq!(memory_usage_kb, 0);
        Ok(())
    }

    #[test]
    fn test_get_memory_stats_available_greater_than_total() -> io::Result<()> {
        let mut mock_provider = MockProcProvider::new();
        mock_provider.expect_get_proc_meminfo().returning(|| {
            Ok(vec![
                "MemTotal:        8000000 kB".to_string(),
                "MemAvailable:    9000000 kB".to_string(), // greater than total
            ])
        });

        let (memory_usage_kb, memory_total_kb) = get_memory_usage_and_total_kb(&mock_provider)?;
        assert_eq!(memory_total_kb, 8000000);
        assert_eq!(memory_usage_kb, 0);
        Ok(())
    }

    #[test]
    fn test_get_memory_stats_io_error() {
        let mut mock_provider = MockProcProvider::new();
        mock_provider
            .expect_get_proc_meminfo()
            .returning(|| Err(io::Error::new(io::ErrorKind::NotFound, "File not found")));

        assert!(get_memory_usage_and_total_kb(&mock_provider).is_err());
    }

    #[test]
    fn test_parse_proc_meminfo_value() {
        assert_eq!(parse_proc_meminfo_value("MemTotal: 8048836 kB"), 8048836);
        assert_eq!(parse_proc_meminfo_value("MemAvailable: 401941 kB"), 401941);
        assert_eq!(parse_proc_meminfo_value("Invalid line"), 0);
        assert_eq!(parse_proc_meminfo_value("MemTotal: invalid kB"), 0);
        assert_eq!(parse_proc_meminfo_value(""), 0);
    }
}
