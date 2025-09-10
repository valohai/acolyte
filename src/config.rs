use crate::consts::ID_ENV_VAR;
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

pub struct Config {
    pub sentry_dsn: Option<String>,
    pub acolyte_id: Uuid,
    pub cpu_sample_interval: Duration,
    pub max_stats_entries: usize,
    pub stat_interval: Duration,
    pub stats_dir: Option<PathBuf>,
    pub cluster_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            sentry_dsn: get_sentry_dsn(),
            acolyte_id: get_or_create_acolyte_id(),
            cpu_sample_interval: get_cpu_sample_interval(),
            max_stats_entries: get_max_stats_entries(),
            stat_interval: get_stat_interval(),
            stats_dir: Some(get_stats_dir()),
            cluster_name: get_cluster_name(),
        }
    }
}

fn get_sentry_dsn() -> Option<String> {
    env::var("SENTRY_DSN").ok()
}

fn get_or_create_acolyte_id() -> Uuid {
    match env::var(ID_ENV_VAR) {
        Ok(id) => Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
        Err(_) => Uuid::new_v4(),
    }
}

fn get_stat_interval() -> Duration {
    let secs = env::var("ACOLYTE_STAT_INTERVAL_MS")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(5000);
    Duration::from_millis(secs)
}

fn get_cpu_sample_interval() -> Duration {
    let ms = env::var("ACOLYTE_CPU_SAMPLE_RATE_MS")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        // 100 ms seems like a common interval to sample CPU usage
        .unwrap_or(100);
    Duration::from_millis(ms)
}

fn get_stats_dir() -> PathBuf {
    env::var("ACOLYTE_STATS_DIR")
        .unwrap_or_else(|_| "/tmp/acolyte/stats".to_string())
        .into()
}

fn get_max_stats_entries() -> usize {
    env::var("ACOLYTE_MAX_STATS_ENTRIES")
        .unwrap_or_else(|_| "12".to_string())
        .parse::<usize>()
        .unwrap_or(12)
}

fn get_cluster_name() -> String {
    env::var("CLUSTER_NAME").unwrap_or_else(|_| "Unknown".to_string())
}
