use super::NvidiaSmiProvider;
use crate::stats::GpuStats;
use std::io;
use tracing::debug;

pub fn get_gpu_stats<P: NvidiaSmiProvider>(provider: &P) -> io::Result<GpuStats> {
    // Format: index, utilization.gpu [%], memory.used [MiB], memory.total [MiB]
    // e.g. "0, 75, 8000, 16000"
    let output = provider.get_nvidia_gpu_stats()?;

    let mut num_gpus = 0;
    let mut total_gpu_usage = 0.0;
    let mut total_memory_usage_kb = 0;
    let mut total_memory_kb = 0;

    for line in output.lines() {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 4 {
            debug!("Skipping malformed line: {}", line);
            continue;
        }

        num_gpus += 1;

        if let Ok(usage) = parts[1].parse::<f64>() {
            total_gpu_usage += usage / 100.0;
        } else {
            debug!("Failed to parse GPU utilization: {}", parts[1]);
        }

        if let Ok(mem_used) = parts[2].parse::<u64>() {
            total_memory_usage_kb += mem_used * 1024; // MiB to KB
        } else {
            debug!("Failed to parse GPU memory used: {}", parts[2]);
        }

        if let Ok(mem_total) = parts[3].parse::<u64>() {
            total_memory_kb += mem_total * 1024; // MiB to KB
        } else {
            debug!("Failed to parse GPU total memory: {}", parts[3]);
        }
    }

    Ok(GpuStats {
        num_gpus,
        gpu_usage: total_gpu_usage,
        memory_usage_kb: total_memory_usage_kb,
        memory_total_kb: total_memory_kb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::nvidia_smi::MockNvidiaSmiProvider;

    #[test]
    fn test_get_gpu_stats_when_available() {
        let mut mock_provider = MockNvidiaSmiProvider::new();
        mock_provider
            .expect_get_nvidia_gpu_stats()
            .returning(|| Ok("0, 75, 8000, 16000\n1, 50, 4000, 16000".to_string()));

        let stats = get_gpu_stats(&mock_provider).unwrap();
        assert_eq!(stats.num_gpus, 2);
        assert_eq!(stats.gpu_usage, 1.25); // 75% + 50% = 125% total
        assert_eq!(stats.memory_usage_kb, 12_288_000); // (8000+4000)*1024
        assert_eq!(stats.memory_total_kb, 32_768_000); // (16000+16000)*1024
    }

    #[test]
    fn test_get_gpu_stats_when_not_available() {
        let mut mock_provider = MockNvidiaSmiProvider::new();
        mock_provider.expect_get_nvidia_gpu_stats().returning(|| {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Failed to run nvidia-smi",
            ))
        });

        let result = get_gpu_stats(&mock_provider);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_gpu_stats_with_empty_output() {
        let mut mock_provider = MockNvidiaSmiProvider::new();
        mock_provider
            .expect_get_nvidia_gpu_stats()
            .returning(|| Ok("".to_string()));

        let stats = get_gpu_stats(&mock_provider).unwrap();
        assert_eq!(stats.num_gpus, 0);
        assert_eq!(stats.gpu_usage, 0.0);
        assert_eq!(stats.memory_usage_kb, 0);
        assert_eq!(stats.memory_total_kb, 0);
    }

    #[test]
    fn test_get_gpu_stats_with_unexpected_output() {
        let mut mock_provider = MockNvidiaSmiProvider::new();
        mock_provider
            .expect_get_nvidia_gpu_stats()
            .returning(|| Ok("this has nothing to do with our stats".to_string()));

        let stats = get_gpu_stats(&mock_provider).unwrap();
        assert_eq!(stats.num_gpus, 0);
        assert_eq!(stats.gpu_usage, 0.0);
        assert_eq!(stats.memory_usage_kb, 0);
        assert_eq!(stats.memory_total_kb, 0);
    }

    #[test]
    fn test_get_gpu_stats_with_malformed_output_line() {
        let mut mock_provider = MockNvidiaSmiProvider::new();
        mock_provider
            .expect_get_nvidia_gpu_stats()
            .returning(|| Ok("0, 75, 8000, 16000\n1, 50, 4000".to_string()));

        // only the first line is valid so total GPU stats reflect that
        let stats = get_gpu_stats(&mock_provider).unwrap();
        assert_eq!(stats.num_gpus, 1);
        assert_eq!(stats.gpu_usage, 0.75);
        assert_eq!(stats.memory_usage_kb, 8_192_000);
        assert_eq!(stats.memory_total_kb, 16_384_000);
    }
}
