use crate::env;
use crate::stats::CpuUsageValue;
use crate::stats::proc::ProcProvider;
use std::io;
use tracing::{debug, warn};

/// Get CPU usage (% of all available CPUS) from the `/proc` filesystem (host-wide)
pub fn get_cpu_usage<R: ProcProvider>(provider: &R) -> io::Result<CpuUsageValue> {
    // CPU measurements from `procfs` are in "jiffies".
    // Jiffy "duration" depends on the kernel configuration, so we sidestep needing to resolve that
    // by calculating the CPU usage as a ratio of time spent being "vacant" (idle + iowait) vs. total time.
    // https://elinux.org/Kernel_Timer_Systems

    // `procfs` values are cumulative since system boot, we need to read
    // the values twice to calculate the CPU usage
    let initial = get_total_cpu_jiffies(provider)?;
    std::thread::sleep(std::time::Duration::from_millis(env::get_cpu_sample_ms()));
    let current = get_total_cpu_jiffies(provider)?;

    let cpu_usage = calculate_cpu_usage(&initial, &current);
    Ok(CpuUsageValue::FromProc(cpu_usage))
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
    let total_cpu_line = &lines[0];
    let jiffies: Vec<u64> = total_cpu_line
        .split_whitespace()
        .skip(1) // skip the "cpu*" prefix
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();

    Ok(jiffies)
}

/// Calculate CPU usage based on two sequential readings from `/proc/stat`
fn calculate_cpu_usage(initial_jiffies: &[u64], current_jiffies: &[u64]) -> f64 {
    // From: https://man7.org/linux/man-pages/man5/proc_stat.5.html
    const IDLE_IDX: usize = 3; // idle is the 4th field
    const IOWAIT_IDX: usize = 4; // iowait is the 5th field
    const MIN_REQUIRED_LEN: usize = IOWAIT_IDX + 1;

    if initial_jiffies.len() < MIN_REQUIRED_LEN {
        debug!(
            "Initial CPU reading is incomplete: expected at least {} fields, got {}",
            MIN_REQUIRED_LEN,
            initial_jiffies.len()
        );
        return 0.0;
    }
    if current_jiffies.len() < MIN_REQUIRED_LEN {
        debug!(
            "Current CPU reading is incomplete: expected at least {} fields, got {}",
            MIN_REQUIRED_LEN,
            current_jiffies.len()
        );
        return 0.0;
    }
    if initial_jiffies.len() != current_jiffies.len() {
        debug!(
            "Initial and current CPU readings have different lengths: {} vs {}",
            initial_jiffies.len(),
            current_jiffies.len()
        );
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

    // calculate "vacant" time (idle + iowait)
    let initial_vacancy = initial_jiffies[IDLE_IDX] + initial_jiffies[IOWAIT_IDX];
    let current_vacancy = current_jiffies[IDLE_IDX] + current_jiffies[IOWAIT_IDX];
    let vacant_delta = current_vacancy.saturating_sub(initial_vacancy);

    debug!("Using proc for CPU usage");
    1.0 - (vacant_delta as f64 / total_delta as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::proc::MockProcProvider;

    #[test]
    fn test_get_total_cpu_jiffies() {
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
        //                                | idle
        //                                v    v iowait
        let initial = vec![100, 200, 300, 400, 500]; // = 1500 jiffies
        let current = vec![110, 220, 330, 440, 550]; // = 1650 jiffies

        let usage = calculate_cpu_usage(&initial, &current);

        // 1650 - 1500 =    150 total time spent (delta)
        //  440 - 400  =     40 time spent idle (delta)
        //  550 - 500  =     50 time spent waiting for I/O (delta)
        //   90 / 150  =    0.6 (idle + iowait)%
        //  1.0 - 0.6 =     0.4 usage%
        assert_eq!(usage, 0.4);
    }
}
