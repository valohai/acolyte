use crate::consts::ID_ENV_VAR;
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

pub struct JsonlToStdoutConfig {
    pub prefix: String,
}

pub struct StatsDirConfig {
    pub dir: PathBuf,
    pub max_stats_entries: usize,
}

pub enum OutputMode {
    JsonlToStdout(JsonlToStdoutConfig),
    StatsDir(StatsDirConfig),
}
pub struct Config {
    pub sentry_dsn: Option<String>,
    pub acolyte_id: Uuid,
    pub cpu_sample_interval: Duration,
    pub stat_interval: Duration,
    pub cluster_name: String,
    pub output_mode: OutputMode,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            sentry_dsn: get_sentry_dsn(),
            acolyte_id: get_or_create_acolyte_id(),
            cpu_sample_interval: get_cpu_sample_interval(),
            stat_interval: get_stat_interval(),
            output_mode: get_output_mode()?,
            cluster_name: get_cluster_name(),
        })
    }
}

fn get_output_mode() -> anyhow::Result<OutputMode> {
    let output_mode = env::var("ACOLYTE_OUTPUT_MODE").ok();
    match output_mode.as_deref() {
        Some("stdout") => {
            let prefix = env::var("ACOLYTE_OUTPUT_PREFIX").unwrap_or_else(|_| "".to_string());
            Ok(OutputMode::JsonlToStdout(JsonlToStdoutConfig { prefix }))
        }
        Some("dir") | None => Ok(OutputMode::StatsDir(StatsDirConfig {
            dir: get_stats_dir(),
            max_stats_entries: get_max_stats_entries(),
        })),
        Some(other) => Err(anyhow::anyhow!("Invalid ACOLYTE_OUTPUT_MODE: {other}.")),
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
