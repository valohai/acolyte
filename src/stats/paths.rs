use crate::stats::CgroupVersion;
use crate::stats::cgroup_v1::CgroupV1MountPoints;
use std::io;
use std::path::{Path, PathBuf};

const MOUNT_POINT_INDEX: usize = 1; // ... in /proc/mounts
const FILESYSTEM_TYPE_INDEX: usize = 2; // ... in /proc/mounts
const FILESYSTEM_OPTIONS_INDEX: usize = 3; // ... in /proc/mounts

/// Return the single mount point for the cgroup v2 unified hierarchy.
///
/// This will most frequently return `/sys/fs/cgroup`, but _can_ be different.
pub fn get_cgroup_v2_mount_point<P: AsRef<Path>>(proc_mounts_path: P) -> io::Result<PathBuf> {
    let content = std::fs::read_to_string(proc_mounts_path)?;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < FILESYSTEM_TYPE_INDEX + 1 {
            continue;
        }
        if parts[FILESYSTEM_TYPE_INDEX] == "cgroup2" {
            // with the v2 unified hierarchy, there is a single mount point for all controllers
            return Ok(PathBuf::from(parts[MOUNT_POINT_INDEX]));
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No cgroup v2 mount point found",
    ))
}

/// Return the mount points for the cgroup v1 hierarchy.
pub fn get_cgroup_v1_mount_points<P: AsRef<Path>>(
    proc_mounts_path: P,
) -> io::Result<CgroupV1MountPoints> {
    let content = std::fs::read_to_string(proc_mounts_path)?;

    let mut v1_points = CgroupV1MountPoints::default();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < FILESYSTEM_OPTIONS_INDEX + 1 {
            continue;
        }

        if parts[FILESYSTEM_TYPE_INDEX] == "cgroup" {
            // the paths may differ, but the controller name(s) are in the options
            // and note that controllers can be co-mounted like `cpu,cpuacct` _usually_ are
            let mount_point = parts[MOUNT_POINT_INDEX];
            let options_str = parts[FILESYSTEM_OPTIONS_INDEX];
            let options: Vec<&str> = options_str.split(',').collect();

            if options.contains(&"cpu") {
                v1_points.cpu = Some(PathBuf::from(mount_point));
            }
            if options.contains(&"cpuacct") {
                v1_points.cpuacct = Some(PathBuf::from(mount_point));
            }
            if options.contains(&"memory") {
                v1_points.memory = Some(PathBuf::from(mount_point));
            }
        }
    }

    Ok(v1_points)
}

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
    fn test_v2_mount_point() -> io::Result<()> {
        let v2_content = "\
overlay / overlay rw,relatime,lowerdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1001/fs,upperdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1543/fs,workdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1543/work,uuid=on,nouserxattr 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
tmpfs /dev tmpfs rw,nosuid,size=65536k,mode=755,inode64 0 0
devpts /dev/pts devpts rw,nosuid,noexec,relatime,gid=5,mode=620,ptmxmode=666 0 0
mqueue /dev/mqueue mqueue rw,nosuid,nodev,noexec,relatime 0 0
sysfs /sys sysfs ro,nosuid,nodev,noexec,relatime 0 0
cgroup /sys/fs/cgroup cgroup2 ro,nosuid,nodev,noexec,relatime,nsdelegate,memory_recursiveprot 0 0
/dev/mapper/vgubuntu-root /etc/hosts ext4 rw,relatime,errors=remount-ro,stripe=32 0 0
/dev/mapper/vgubuntu-root /dev/termination-log ext4 rw,relatime,errors=remount-ro,stripe=32 0 0
/dev/mapper/vgubuntu-root /etc/hostname ext4 rw,relatime,errors=remount-ro,stripe=32 0 0
/dev/mapper/vgubuntu-root /etc/resolv.conf ext4 rw,relatime,errors=remount-ro,stripe=32 0 0
shm /dev/shm tmpfs rw,relatime,size=65536k,inode64 0 0
tmpfs /run/secrets/kubernetes.io/serviceaccount tmpfs ro,relatime,size=131072k,inode64,noswap 0 0
overlay /sys/devices/virtual/dmi/id/product_name overlay ro,relatime,lowerdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1001/fs,upperdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1543/fs,workdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1543/work,uuid=on,nouserxattr 0 0
overlay /sys/devices/virtual/dmi/id/product_uuid overlay ro,relatime,lowerdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1001/fs,upperdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1543/fs,workdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1543/work,uuid=on,nouserxattr 0 0
overlay /sys/devices/virtual/dmi/id/product_uuid overlay ro,relatime,lowerdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1001/fs,upperdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1543/fs,workdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/1543/work,uuid=on,nouserxattr 0 0
proc /proc/bus proc ro,nosuid,nodev,noexec,relatime 0 0
proc /proc/fs proc ro,nosuid,nodev,noexec,relatime 0 0
proc /proc/irq proc ro,nosuid,nodev,noexec,relatime 0 0
proc /proc/sys proc ro,nosuid,nodev,noexec,relatime 0 0
proc /proc/sysrq-trigger proc ro,nosuid,nodev,noexec,relatime 0 0
tmpfs /proc/asound tmpfs ro,relatime,inode64 0 0
tmpfs /proc/acpi tmpfs ro,relatime,inode64 0 0
tmpfs /proc/kcore tmpfs rw,nosuid,size=65536k,mode=755,inode64 0 0
tmpfs /proc/keys tmpfs rw,nosuid,size=65536k,mode=755,inode64 0 0
tmpfs /proc/latency_stats tmpfs rw,nosuid,size=65536k,mode=755,inode64 0 0
tmpfs /proc/timer_list tmpfs rw,nosuid,size=65536k,mode=755,inode64 0 0
tmpfs /proc/scsi tmpfs ro,relatime,inode64 0 0
tmpfs /sys/firmware tmpfs ro,relatime,inode64 0 0
tmpfs /sys/devices/virtual/powercap tmpfs ro,relatime,inode64 0 0";

        let mut v2_file = NamedTempFile::new()?;
        v2_file.write_all(v2_content.as_bytes())?;

        let mount_point = get_cgroup_v2_mount_point(v2_file)?;
        assert_eq!(mount_point, PathBuf::from("/sys/fs/cgroup"));
        Ok(())
    }

    #[test]
    fn test_no_v2_mount_point() -> io::Result<()> {
        let no_cgroup_content = "\
overlay / overlay rw,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
tmpfs /dev tmpfs rw,nosuid,size=65536k,mode=755,inode64 0 0";

        let mut no_cgroup_file = NamedTempFile::new()?;
        no_cgroup_file.write_all(no_cgroup_content.as_bytes())?;

        assert!(get_cgroup_v2_mount_point(no_cgroup_file).is_err());
        Ok(())
    }

    #[test]
    fn test_v1_mount_points() -> io::Result<()> {
        let v1_content = "\
overlay / overlay rw,relatime,lowerdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/3753/fs,upperdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/3760/fs,workdir=/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots/3760/work 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
tmpfs /dev tmpfs rw,nosuid,size=65536k,mode=755 0 0
devpts /dev/pts devpts rw,nosuid,noexec,relatime,gid=5,mode=620,ptmxmode=666 0 0
mqueue /dev/mqueue mqueue rw,nosuid,nodev,noexec,relatime 0 0
sysfs /sys sysfs ro,nosuid,nodev,noexec,relatime 0 0
tmpfs /sys/fs/cgroup tmpfs rw,nosuid,nodev,noexec,relatime,mode=755 0 0
cgroup /sys/fs/cgroup/systemd cgroup ro,nosuid,nodev,noexec,relatime,xattr,release_agent=/usr/lib/systemd/systemd-cgroups-agent,name=systemd 0 0
cgroup /sys/fs/cgroup/perf_event cgroup ro,nosuid,nodev,noexec,relatime,perf_event 0 0
cgroup /sys/fs/cgroup/cpu,cpuacct cgroup ro,nosuid,nodev,noexec,relatime,cpu,cpuacct 0 0
cgroup /sys/fs/cgroup/memory cgroup ro,nosuid,nodev,noexec,relatime,memory 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio cgroup ro,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0
cgroup /sys/fs/cgroup/devices cgroup ro,nosuid,nodev,noexec,relatime,devices 0 0
cgroup /sys/fs/cgroup/hugetlb cgroup ro,nosuid,nodev,noexec,relatime,hugetlb 0 0
cgroup /sys/fs/cgroup/freezer cgroup ro,nosuid,nodev,noexec,relatime,freezer 0 0
cgroup /sys/fs/cgroup/cpuset cgroup ro,nosuid,nodev,noexec,relatime,cpuset 0 0
cgroup /sys/fs/cgroup/pids cgroup ro,nosuid,nodev,noexec,relatime,pids 0 0
cgroup /sys/fs/cgroup/blkio cgroup ro,nosuid,nodev,noexec,relatime,blkio 0 0
/dev/nvme0n1p1 /etc/hosts xfs rw,noatime,attr2,inode64,logbufs=8,logbsize=32k,noquota 0 0
/dev/nvme0n1p1 /dev/termination-log xfs rw,noatime,attr2,inode64,logbufs=8,logbsize=32k,noquota 0 0
/dev/nvme0n1p1 /etc/hostname xfs rw,noatime,attr2,inode64,logbufs=8,logbsize=32k,noquota 0 0
/dev/nvme0n1p1 /etc/resolv.conf xfs rw,noatime,attr2,inode64,logbufs=8,logbsize=32k,noquota 0 0
shm /dev/shm tmpfs rw,nosuid,nodev,noexec,relatime,size=65536k 0 0
tmpfs /run/secrets/kubernetes.io/serviceaccount tmpfs ro,relatime,size=131072k 0 0
proc /proc/bus proc ro,nosuid,nodev,noexec,relatime 0 0
proc /proc/fs proc ro,nosuid,nodev,noexec,relatime 0 0
proc /proc/irq proc ro,nosuid,nodev,noexec,relatime 0 0
proc /proc/sys proc ro,nosuid,nodev,noexec,relatime 0 0
proc /proc/sysrq-trigger proc ro,nosuid,nodev,noexec,relatime 0 0
tmpfs /proc/acpi tmpfs ro,relatime 0 0
tmpfs /proc/kcore tmpfs rw,nosuid,size=65536k,mode=755 0 0
tmpfs /proc/keys tmpfs rw,nosuid,size=65536k,mode=755 0 0
tmpfs /proc/latency_stats tmpfs rw,nosuid,size=65536k,mode=755 0 0
tmpfs /proc/timer_list tmpfs rw,nosuid,size=65536k,mode=755 0 0
tmpfs /proc/sched_debug tmpfs rw,nosuid,size=65536k,mode=755 0 0
tmpfs /sys/firmware tmpfs ro,relatime 0 0";

        let mut v1_file = NamedTempFile::new()?;
        v1_file.write_all(v1_content.as_bytes())?;

        let mp = get_cgroup_v1_mount_points(v1_file)?;
        assert_eq!(mp.cpu, Some("/sys/fs/cgroup/cpu,cpuacct".into()));
        assert_eq!(mp.cpuacct, Some("/sys/fs/cgroup/cpu,cpuacct".into()));
        assert_eq!(mp.memory, Some("/sys/fs/cgroup/memory".into()));
        Ok(())
    }

    #[test]
    fn test_v1_mount_points_separate() -> io::Result<()> {
        let v1_content = "\
cgroup /sys/fs/cgroup/cpu cgroup ro,nosuid,nodev,noexec,relatime,cpu 0 0
cgroup /sys/fs/cgroup/cpuacct cgroup ro,nosuid,nodev,noexec,relatime,cpuacct 0 0
cgroup /sys/fs/cgroup/memory cgroup ro,nosuid,nodev,noexec,relatime,memory 0 0";

        let mut v1_file = NamedTempFile::new()?;
        v1_file.write_all(v1_content.as_bytes())?;

        let mp = get_cgroup_v1_mount_points(v1_file)?;
        assert_eq!(mp.cpu, Some(PathBuf::from("/sys/fs/cgroup/cpu")));
        assert_eq!(mp.cpuacct, Some(PathBuf::from("/sys/fs/cgroup/cpuacct")));
        assert_eq!(mp.memory, Some(PathBuf::from("/sys/fs/cgroup/memory")));
        Ok(())
    }

    #[test]
    fn test_v1_mount_points_only_memory() -> io::Result<()> {
        let v1_content = "\
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
cgroup /sys/fs/cgroup/memory cgroup ro,nosuid,nodev,noexec,relatime,memory 0 0
tmpfs /dev tmpfs rw,nosuid,size=65536k,mode=755 0 0";

        let mut v1_file = NamedTempFile::new()?;
        v1_file.write_all(v1_content.as_bytes())?;

        let mp = get_cgroup_v1_mount_points(v1_file)?;
        assert_eq!(mp.cpu, None);
        assert_eq!(mp.cpuacct, None);
        assert_eq!(mp.memory, Some(PathBuf::from("/sys/fs/cgroup/memory")));
        Ok(())
    }

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
