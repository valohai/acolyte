mod gpu_stats;

pub use gpu_stats::get_gpu_stats;
use std::io;
use std::process::Command;

#[cfg(test)]
use mockall::automock;
use tracing::debug;

#[cfg_attr(test, automock)]
pub trait NvidiaSmiProvider {
    fn get_nvidia_gpu_stats(&self) -> io::Result<String>;
}

pub struct NvidiaSmiExecutor;

impl NvidiaSmiExecutor {
    pub fn new() -> Self {
        Self {}
    }
}

impl NvidiaSmiProvider for NvidiaSmiExecutor {
    fn get_nvidia_gpu_stats(&self) -> io::Result<String> {
        let output = Command::new("nvidia-smi")
            .args([
                "--query-gpu=index,utilization.gpu,memory.used,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Failed to run nvidia-smi: {e}"),
                )
            })?;

        if !output.status.success() {
            return Err(io::Error::other(format!(
                "nvidia-smi exited with non-zero status: {}. stderr: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        debug!("Using nvidia-smi for GPU stats"); // report use here as we don't check for nvidia-smi availability
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
