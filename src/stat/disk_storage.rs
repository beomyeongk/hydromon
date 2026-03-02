use crate::db::DiskStorage;
use std::ffi::CString;
use std::io;
use std::mem;

pub struct DiskStorageStats;

impl DiskStorageStats {
    pub fn new() -> Self {
        DiskStorageStats
    }

    pub fn update(&self, now_in_secs: i64, mounts: &[String]) -> io::Result<Vec<DiskStorage>> {
        let mut results = Vec::new();

        for mount_point in mounts {
            let path = CString::new(mount_point.as_str())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

            let mut stats: libc::statvfs = unsafe { mem::zeroed() };

            let result = unsafe { libc::statvfs(path.as_ptr(), &mut stats) };

            if result == 0 {
                let bsize = stats.f_bsize as u64; // Block size
                let blocks = stats.f_blocks as u64; // Total blocks
                let bfree = stats.f_bfree as u64; // Free blocks

                // Total size in 16 KiB units (bytes / 16384)
                let total = (blocks * bsize) / 16384;
                // Used size in 16 KiB units
                let free = (bfree * bsize) / 16384;
                let used = total.saturating_sub(free);

                // Inodes
                let total_inodes = stats.f_files as u64; // Total inodes
                let free_inodes = stats.f_ffree as u64; // Free inodes
                let used_inodes = total_inodes.saturating_sub(free_inodes);

                results.push(DiskStorage {
                    timestamp: now_in_secs,
                    mount_point: mount_point.clone(),
                    total: total as u32,
                    used: used as u32,
                    num_inodes: used_inodes as i64,
                });
            } else {
                eprintln!(
                    "Error calling statvfs for {}: {}",
                    mount_point,
                    io::Error::last_os_error()
                );
            }
        }

        Ok(results)
    }
}
