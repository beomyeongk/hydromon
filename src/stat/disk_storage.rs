use crate::db::{DiskStorage, NameMapper};
use std::ffi::CString;
use std::io;
use std::mem;

pub struct DiskStorageStats;

impl DiskStorageStats {
    pub fn new() -> Self {
        DiskStorageStats
    }

    pub fn update(
        &self,
        now_in_secs: i64,
        mounts: &[String],
        name_mapper: &NameMapper,
    ) -> io::Result<Vec<DiskStorage>> {
        let mut results = Vec::new();

        for mount_point in mounts {
            let path = CString::new(mount_point.as_str())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

            let mut stats: libc::statvfs = unsafe { mem::zeroed() };

            let result = unsafe { libc::statvfs(path.as_ptr(), &mut stats) };

            if result == 0 {
                let bsize = stats.f_bsize as u64;
                let blocks = stats.f_blocks as u64;
                let bfree = stats.f_bfree as u64;

                let total = (blocks * bsize) / 16384;
                let free = (bfree * bsize) / 16384;
                let used = total.saturating_sub(free);

                let total_inodes = stats.f_files as u64;
                let free_inodes = stats.f_ffree as u64;
                let used_inodes = total_inodes.saturating_sub(free_inodes);

                results.push(DiskStorage {
                    timestamp: now_in_secs,
                    name_id: name_mapper.get(mount_point),
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
