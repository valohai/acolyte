use std::env;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

pub const MAX_RUN_ATTEMPTS: u8 = 5;
pub const RESTART_DELAY_SECS: u64 = 10;

pub const RESTART_ENV_VAR: &str = "ACOLYTE_RESTART";
pub const ID_ENV_VAR: &str = "ACOLYTE_ID";

pub fn get_sentry_dsn() -> Option<String> {
    env::var("SENTRY_DSN").ok()
}

pub fn get_cluster_name() -> String {
    env::var("CLUSTER_NAME").unwrap_or_else(|_| "Unknown".to_string())
}

pub fn get_restart_count() -> u8 {
    env::var(RESTART_ENV_VAR)
        .ok()
        .and_then(|count_str| count_str.parse::<u8>().ok())
        .unwrap_or(0)
}

pub fn get_or_create_acolyte_id() -> Uuid {
    match env::var(ID_ENV_VAR) {
        Ok(id) => Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
        Err(_) => Uuid::new_v4(),
    }
}

pub fn get_stat_interval() -> Duration {
    let secs = env::var("ACOLYTE_STAT_INTERVAL_MS")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(5000);
    Duration::from_millis(secs)
}

pub fn get_cpu_sample_ms() -> u64 {
    // 100 ms seems like a common interval to sample CPU usage
    env::var("ACOLYTE_CPU_SAMPLE_RATE_MS")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(100)
}

pub fn get_stats_dir() -> PathBuf {
    env::var("ACOLYTE_STATS_DIR")
        .unwrap_or_else(|_| "/tmp/acolyte/stats".to_string())
        .into()
}

pub fn get_max_stats_entries() -> usize {
    env::var("ACOLYTE_MAX_STATS_ENTRIES")
        .unwrap_or_else(|_| "12".to_string())
        .parse::<usize>()
        .unwrap_or(12)
}
