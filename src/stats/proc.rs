use crate::stats::{CpuStats, MemoryStats};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use tracing::warn;

/// Get CPU stats from the `/proc` filesystem (host-wide)
pub fn get_cpu_stats() -> io::Result<CpuStats> {
    // values are cumulative, we need to read the values twice
    // to calculate the CPU usage over a time interval, 100 ms seems common
    let initial = read_cpu_stat()?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    let current = read_cpu_stat()?;

    let usage_percentage = calculate_cpu_usage(&initial, &current);
    let num_cpus = get_cpu_count()? as f64;

    // scale the usage by the number of CPUs so that:
    // 1.0 = 100% of a single CPU
    // 2.0 = 100% of two CPUs, etc.
    let cpu_usage = usage_percentage * num_cpus;

    Ok(CpuStats {
        cpu_usage,
        num_cpus,
    })
}

/// Get memory stats from the `/proc` filesystem (host-wide)
pub fn get_memory_stats() -> io::Result<MemoryStats> {
    let file = File::open("/proc/meminfo")?;
    let reader = BufReader::new(file);

    let mut memory_total_kb = 0;
    let mut available_kb = 0;

    for line in reader.lines() {
        let line = line?;

        if line.starts_with("MemTotal:") {
            memory_total_kb = parse_proc_meminfo_value(&line);
        } else if line.starts_with("MemAvailable:") {
            available_kb = parse_proc_meminfo_value(&line);
        }

        if memory_total_kb > 0 && available_kb > 0 {
            break;
        }
    }

    let memory_usage_kb = if available_kb <= memory_total_kb {
        memory_total_kb - available_kb
    } else {
        0
    };

    Ok(MemoryStats {
        memory_usage_kb,
        memory_total_kb,
    })
}

fn get_cpu_count() -> io::Result<u32> {
    let file = File::open("/proc/stat")?;
    let reader = BufReader::new(file);

    // count lines starting with "cpu" but exclude the first one, which is the total stats
    let mut count = 0;
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("cpu") && !line.starts_with("cpu ") {
            count += 1;
        }
    }

    Ok(count)
}

fn read_cpu_stat() -> io::Result<Vec<u64>> {
    let file = File::open("/proc/stat")?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    // parse the CPU values that look like this:
    // cpu10 218029 176 51746 1932122 3466 0 2149 0 0 0
    // https://man7.org/linux/man-pages/man5/proc_stat.5.html
    let values: Vec<u64> = first_line
        .split_whitespace()
        .skip(1) // skip the "cpu*" prefix
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();

    Ok(values)
}

/// Calculate CPU usage based on two sequential readings from `/proc/stat`
fn calculate_cpu_usage(initial: &[u64], current: &[u64]) -> f64 {
    // there are like 10 different values how CPU spent it's time, but
    // we are _mainly_ interested in the idle time, which is the 4th value,
    // don't be so strict about the other values
    // https://man7.org/linux/man-pages/man5/proc_stat.5.html
    const IDLE_IDX: usize = 3;
    const MIN_REQUIRED_LEN: usize = IDLE_IDX + 1;

    if initial.len() < MIN_REQUIRED_LEN {
        warn!("Initial CPU reading is incomplete");
        return 0.0;
    }
    if current.len() < MIN_REQUIRED_LEN {
        warn!("Current CPU reading is incomplete");
        return 0.0;
    }
    if initial.len() != current.len() {
        warn!("Initial and current CPU readings have different lengths");
        return 0.0;
    }

    let initial_total: u64 = initial.iter().sum();
    let current_total: u64 = current.iter().sum();

    let total_delta = current_total.saturating_sub(initial_total);
    if total_delta == 0 {
        // improbable, but possible
        warn!("CPU total time delta is zero - measurement interval may be too short");
        return 0.0;
    }

    let idle_delta = current[IDLE_IDX].saturating_sub(initial[IDLE_IDX]);

    let cpu_usage = 1.0 - (idle_delta as f64 / total_delta as f64);

    cpu_usage
}

fn parse_proc_meminfo_value(line: &str) -> u64 {
    // most /proc/meminfo lines look like this:
    // MemTotal: 8048836 kB
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}
