use procfs::{Meminfo, Current};
use std::io;
use crate::db::MemoryUsage;

pub struct MemoryUsageStats;

impl MemoryUsageStats {
    pub fn new() -> Self {
        MemoryUsageStats
    }

    pub fn collect(&self, now_in_secs: i64) -> io::Result<MemoryUsage> {
        let meminfo = Meminfo::current().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(MemoryUsage {
            timestamp: now_in_secs,
            total: (meminfo.mem_total / 1024) as u32,
            free: (meminfo.mem_free / 1024) as u32,
            available: (meminfo.mem_available.unwrap_or(0) / 1024) as u32,
            buffers: (meminfo.buffers / 1024) as u32,
            cached: (meminfo.cached / 1024) as u32,
            swap_total: (meminfo.swap_total / 1024) as u32,
            swap_usage: ((meminfo.swap_total - meminfo.swap_free) / 1024) as u32,
        })
    }
}
