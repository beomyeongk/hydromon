use crate::db::SysSummary;
use std::fs;
use std::io;

pub struct SysSummaryStats {}

impl SysSummaryStats {
    pub fn new() -> Self {
        Self {}
    }

    pub fn collect(&self, timestamp: i64) -> io::Result<SysSummary> {
        let uptime_str = fs::read_to_string("/proc/uptime")?;
        let uptime: u32 = uptime_str
            .split_whitespace()
            .next()
            .and_then(|s| s.split('.').next())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let loadavg_str = fs::read_to_string("/proc/loadavg")?;
        let mut parts = loadavg_str.split_whitespace();
        let load_avg_1m: u32 = parts
            .next()
            .and_then(|s| s.parse::<f32>().ok())
            .map(|f| (f * 100.0) as u32)
            .unwrap_or(0);

        let total_tasks: u32 = parts
            .nth(2)
            .and_then(|s| s.split('/').nth(1))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let file_nr_str = fs::read_to_string("/proc/sys/fs/file-nr")?;
        let num_fds: u32 = file_nr_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        Ok(SysSummary {
            timestamp,
            uptime,
            total_tasks,
            load_avg_1m,
            num_fds,
        })
    }
}
