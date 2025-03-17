use crate::stats::CgroupVersion;
use std::io;
use std::path::Path;

/// Detect the cgroup version of a process based on `/proc/[self|pid]/cgroup`.
///
/// Mostly used with the `/proc/self/cgroup`, but support other processes with `/proc/[pid]/cgroup` as well.
pub fn detect_cgroup_version<P: AsRef<Path>>(self_cgroup_path: P) -> io::Result<CgroupVersion> {
    let content = std::fs::read_to_string(self_cgroup_path)?;
    match content.lines().count() {
        0 => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid proc self cgroup file format: {}", content),
        )),
        1 => Ok(CgroupVersion::V2),
        _ => Ok(CgroupVersion::V1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_detect_v2() -> io::Result<()> {
        let mut v2_file = NamedTempFile::new()?;
        v2_file.write_all("0::/".as_bytes())?;

        assert_eq!(detect_cgroup_version(v2_file)?, CgroupVersion::V2);
        Ok(())
    }

    #[test]
    fn test_detect_v1() -> io::Result<()> {
        let v1_content = "\
11:blkio:/kubepods.slice/...
10:pids:/kubepods.slice/...
4:memory:/kubepods.slice/...
1:name=systemd:/kubepods.slice/...";
        let mut v1_file = NamedTempFile::new()?;
        v1_file.write_all(v1_content.as_bytes())?;

        assert_eq!(detect_cgroup_version(v1_file)?, CgroupVersion::V1);
        Ok(())
    }

    #[test]
    fn test_detect_probably_not_cgroup_managed() {
        assert!(detect_cgroup_version("/this/do/not/exist").is_err());
    }
}
