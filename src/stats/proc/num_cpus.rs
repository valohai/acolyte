use crate::stats::proc::ProcProvider;
use std::io;
use tracing::debug;

/// Get the number of CPUs from the `/proc` filesystem (host-wide)
pub fn get_num_cpus<R: ProcProvider>(provider: &R) -> io::Result<f64> {
    let lines = provider.get_proc_stat()?;

    // skip the line with `cpu` without a number, that is the sum of all CPUs
    // Kubernetes CPU "counts" (limits) can be fractional so we return a float
    let count = lines
        .iter()
        .filter(|line| line.starts_with("cpu") && !line.starts_with("cpu "))
        .count() as f64;

    debug!("Using proc for CPU count");
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::proc::MockProcProvider;

    #[test]
    fn test_get_num_cpus() {
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

        assert_eq!(get_num_cpus(&mock_provider).unwrap(), 4.0);
    }
}
