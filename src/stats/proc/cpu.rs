use crate::stats::proc::ProcProvider;
use crate::stats::CpuStats;
use std::io;
use tracing::{debug, warn};

/// Get CPU stats from the `/proc` filesystem (host-wide)
pub fn get_cpu_stats<R: ProcProvider>(provider: &R) -> io::Result<CpuStats> {
    // values are cumulative, we need to read the values twice
    // to calculate the CPU usage over a time interval (delta),
    // 100 ms seems common
    let initial = get_total_cpu_jiffies(provider)?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    let current = get_total_cpu_jiffies(provider)?;

    let usage_percentage = calculate_cpu_usage(&initial, &current);
    let num_cpus = get_cpu_count(provider)? as f64; // NB: Kubernetes cores can be fractional

    // scale the usage by the number of CPUs so that:
    // 1.0 = 100% of a single CPU
    // 2.0 = 100% of two CPUs, etc.
    let cpu_usage = usage_percentage * num_cpus;

    Ok(CpuStats {
        cpu_usage,
        num_cpus,
    })
}

fn get_total_cpu_jiffies<R: ProcProvider>(provider: &R) -> io::Result<Vec<u64>> {
    let lines = provider.get_proc_stat()?;
    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "No stat data from proc provider",
        ));
    }

    // we only care about the first line, which is the total CPU stats
    // as we don't report CPU stats per core
    let first_line = &lines[0];
    let jiffies: Vec<u64> = first_line
        .split_whitespace()
        .skip(1) // skip the "cpu*" prefix
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();

    Ok(jiffies)
}

/// Calculate CPU usage based on two sequential readings from `/proc/stat`
fn calculate_cpu_usage(initial_jiffies: &[u64], current_jiffies: &[u64]) -> f64 {
    // there are like 10 different values how CPU spent it's time, but
    // we are _mainly_ interested in the idle time, which is the 4th value,
    // don't be so strict about the other values
    // https://man7.org/linux/man-pages/man5/proc_stat.5.html
    const IDLE_IDX: usize = 3;
    const MIN_REQUIRED_LEN: usize = IDLE_IDX + 1;

    if initial_jiffies.len() < MIN_REQUIRED_LEN {
        debug!("Initial CPU reading is incomplete");
        return 0.0;
    }
    if current_jiffies.len() < MIN_REQUIRED_LEN {
        debug!("Current CPU reading is incomplete");
        return 0.0;
    }
    if initial_jiffies.len() != current_jiffies.len() {
        debug!("Initial and current CPU readings have different lengths");
        return 0.0;
    }

    let initial_total: u64 = initial_jiffies.iter().sum();
    let current_total: u64 = current_jiffies.iter().sum();

    let total_delta = current_total.saturating_sub(initial_total);
    if total_delta == 0 {
        // improbable, but possible
        warn!("CPU total time delta is zero - measurement interval may be too short?");
        return 0.0;
    }

    let idle_delta = current_jiffies[IDLE_IDX].saturating_sub(initial_jiffies[IDLE_IDX]);

    let cpu_usage = 1.0 - (idle_delta as f64 / total_delta as f64);

    cpu_usage
}

fn get_cpu_count<R: ProcProvider>(reader: &R) -> io::Result<u32> {
    let lines = reader.get_proc_stat()?;

    // count lines starting with "cpu" but exclude the first one, which is the total stats
    let mut count = 0;
    for line in &lines {
        if line.starts_with("cpu") && !line.starts_with("cpu ") {
            count += 1;
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::proc::MockProcProvider;

    #[test]
    fn test_get_cpu_count() {
        let mut mock_provider = MockProcProvider::new();
        mock_provider.expect_get_proc_stat().returning(|| {
            Ok(vec![
                "cpu  1016173 37036 291183 13457001 28111 0 9511 0 0 0".to_string(),
                "cpu0 198607 6779 63175 1870456 4023 0 4291 0 0 0".to_string(),
                "cpu1 194475 6677 61910 1868513 7087 0 2083 0 0 0".to_string(),
                "cpu2 189167 6556 58132 1870369 5846 0 1428 0 0 0".to_string(),
                "cpu3 196374 6876 58228 1864699 4843 0 1002 0 0 0".to_string(),
                "intr 60444506 7 0 0 0 4517864 0 0 0 1 0 0 0 0 0".to_string(),
                "ctxt 146138886".to_string(),
                "btime 1708345562".to_string(),
            ])
        });

        assert_eq!(get_cpu_count(&mock_provider).unwrap(), 4);
    }

    #[test]
    fn test_read_cpu_stat() {
        let mut mock_provider = MockProcProvider::new();
        mock_provider.expect_get_proc_stat().returning(|| {
            Ok(vec![
                "cpu  1016173 37036 291183 13457001 28111 0 9511 0 0 0".to_string(),
                "cpu0 198607 6779 63175 1870456 4023 0 4291 0 0 0".to_string(),
            ])
        });

        let jiffies = get_total_cpu_jiffies(&mock_provider).unwrap();
        let expected = vec![1016173, 37036, 291183, 13457001, 28111, 0, 9511, 0, 0, 0];
        assert_eq!(jiffies, expected);
    }

    #[test]
    fn test_calculate_cpu_usage() {
        //                                v idle time at index 3
        let initial = vec![100, 200, 300, 400, 500]; // = 1500 jiffies
        let current = vec![110, 220, 330, 440, 550]; // = 1650 jiffies

        let usage = calculate_cpu_usage(&initial, &current);

        // 1650 - 1500 =      150 total spent time (delta)
        //  440 - 400  =       40 time spent idle (delta)
        //   40 / 150  =   0.2667 idle%
        //  1.0 - 0.2667 = 0.7333 usage%
        assert!((usage - 0.7333).abs() < 0.0001);
    }
}
