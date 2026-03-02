use crate::db::{DiskIo, NameMapper};
use procfs::diskstats;
use std::collections::HashMap;
use std::io;
use std::time::Instant;

struct PrevStat {
    stat: procfs::DiskStat,
    timestamp: Instant,
}

pub struct DiskIoStats {
    prev_stats: HashMap<String, PrevStat>,
}

impl DiskIoStats {
    pub fn new() -> Self {
        DiskIoStats {
            prev_stats: HashMap::new(),
        }
    }

    pub fn update(
        &mut self,
        now_in_secs: i64,
        target_devices: &[String],
        name_mapper: &NameMapper,
    ) -> io::Result<Vec<DiskIo>> {
        let current_diskstats = diskstats().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let now = Instant::now();
        let mut results = Vec::new();

        for stat in current_diskstats {
            if !target_devices.contains(&stat.name) {
                continue;
            }

            if let Some(prev) = self.prev_stats.get(&stat.name) {
                let time_delta = now.duration_since(prev.timestamp);
                let time_delta_secs = time_delta.as_secs_f64();
                let time_delta_ms = time_delta.as_millis() as f64;

                if time_delta_secs > 0.0 {
                    // Deltas
                    let read_sectors_delta =
                        stat.sectors_read.saturating_sub(prev.stat.sectors_read);
                    let write_sectors_delta = stat
                        .sectors_written
                        .saturating_sub(prev.stat.sectors_written);
                    let reads_completed_delta = stat.reads.saturating_sub(prev.stat.reads);
                    let writes_completed_delta = stat.writes.saturating_sub(prev.stat.writes);
                    let time_reading_delta =
                        stat.time_reading.saturating_sub(prev.stat.time_reading);
                    let time_writing_delta =
                        stat.time_writing.saturating_sub(prev.stat.time_writing);
                    let time_spent_doing_io_delta = stat
                        .time_in_progress
                        .saturating_sub(prev.stat.time_in_progress);
                    let weighted_time_doing_io_delta = stat
                        .weighted_time_in_progress
                        .saturating_sub(prev.stat.weighted_time_in_progress);

                    // r_kbps, w_kbps (KiB/s) -> INTEGER
                    let r_kbps =
                        (read_sectors_delta as f64 * 512.0 / 1024.0 / time_delta_secs) as u32;
                    let w_kbps =
                        (write_sectors_delta as f64 * 512.0 / 1024.0 / time_delta_secs) as u32;

                    let total_iops =
                        (reads_completed_delta + writes_completed_delta) as f64 / time_delta_secs;
                    let iops = (total_iops / 256.0) as u32;

                    let r_await = if reads_completed_delta > 0 {
                        (time_reading_delta as f64 / reads_completed_delta as f64) as u32
                    } else {
                        0
                    };
                    let w_await = if writes_completed_delta > 0 {
                        (time_writing_delta as f64 / writes_completed_delta as f64) as u32
                    } else {
                        0
                    };

                    let aqu_sz_raw = weighted_time_doing_io_delta as f64 / time_delta_ms;
                    let aqu_sz = (aqu_sz_raw * 16.0) as u32;

                    let util_percent = (time_spent_doing_io_delta as f64 / time_delta_ms) * 100.0;
                    let util = (util_percent * 16.0) as u32;

                    results.push(DiskIo {
                        timestamp: now_in_secs,
                        name_id: name_mapper.get(&stat.name),
                        r_kbps,
                        w_kbps,
                        r_await,
                        w_await,
                        aqu_sz,
                        util,
                        iops,
                    });
                }
            }

            self.prev_stats.insert(
                stat.name.clone(),
                PrevStat {
                    stat,
                    timestamp: now,
                },
            );
        }

        Ok(results)
    }
}
