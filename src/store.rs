use crate::env;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error};

#[derive(Serialize, Debug)]
pub struct StatsEntry {
    pub time: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_cpus: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_usage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_usage_kb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_total_kb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_gpus: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_usage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_memory_usage_kb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_memory_total_kb: Option<u64>,
}

impl StatsEntry {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        StatsEntry {
            time: now,
            num_cpus: None,
            cpu_usage: None,
            memory_usage_kb: None,
            memory_total_kb: None,
            num_gpus: None,
            gpu_usage: None,
            gpu_memory_usage_kb: None,
            gpu_memory_total_kb: None,
        }
    }
}

pub fn write_stats_entry(entry: StatsEntry) -> io::Result<()> {
    let dir_path = env::get_stats_dir();
    ensure_dir_exists(&dir_path)?;

    let timestamp_ms = (entry.time * 1000.0) as u64;
    let filename = format!("stats-{}.json", timestamp_ms);
    let file_path = dir_path.join(filename);

    let as_json = serde_json::to_string_pretty(&entry)?;
    let mut json_file = File::create(file_path)?;
    json_file.write_all(as_json.as_bytes())?;

    clean_up_old_stats_entries(&dir_path)?;
    Ok(())
}

fn ensure_dir_exists(dir_path: &Path) -> io::Result<()> {
    if !dir_path.exists() {
        debug!("Creating stats directory: {:?}", dir_path);
        fs::create_dir_all(dir_path)?;
    }
    Ok(())
}

fn clean_up_old_stats_entries(dir_path: &Path) -> io::Result<()> {
    let max_entries = env::get_max_stats_entries();

    let mut entries: Vec<PathBuf> = fs::read_dir(dir_path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path.extension().map_or(false, |ext| ext == "json")
                && path
                    .file_name()
                    .map_or(false, |name| name.to_string_lossy().starts_with("stats-"))
        })
        .collect();

    if entries.len() <= max_entries {
        return Ok(());
    }

    // Unix timestamp is in the name, so we can sort by that
    entries.sort();

    let to_remove = entries.len() - max_entries;
    for path in entries.into_iter().take(to_remove) {
        debug!("Removing old stats entry: {:?}", path);
        if let Err(e) = fs::remove_file(&path) {
            error!("Failed to remove old stats entry: {:?}", e);
        }
    }

    Ok(())
}
