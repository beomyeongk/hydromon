use crate::db::CpuUsage;
use procfs::CurrentSI;
use std::io;

pub struct CpuUsageStats {
    prev_cpu_times: Option<Vec<procfs::CpuTime>>,
}

impl CpuUsageStats {
    pub fn new() -> Self {
        CpuUsageStats {
            prev_cpu_times: None,
        }
    }

    pub fn update(&mut self, now_in_secs: i64) -> io::Result<Option<CpuUsage>> {
        let stats =
            procfs::KernelStats::current().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // stats.cpu_time provides per-core stats
        let current_cpu_times = stats.cpu_time;
        let mut usages = Vec::new();
        let mut result = None;

        if let Some(prev_times) = &self.prev_cpu_times {
            // Ensure we have the same number of cores
            if prev_times.len() == current_cpu_times.len() {
                for (prev, current) in prev_times.iter().zip(current_cpu_times.iter()) {
                    let prev_idle = prev.idle + prev.iowait.unwrap_or(0);
                    let curr_idle = current.idle + current.iowait.unwrap_or(0);

                    let prev_total = prev.user
                        + prev.nice
                        + prev.system
                        + prev.idle
                        + prev.iowait.unwrap_or(0)
                        + prev.irq.unwrap_or(0)
                        + prev.softirq.unwrap_or(0)
                        + prev.steal.unwrap_or(0)
                        + prev.guest.unwrap_or(0)
                        + prev.guest_nice.unwrap_or(0);

                    let curr_total = current.user
                        + current.nice
                        + current.system
                        + current.idle
                        + current.iowait.unwrap_or(0)
                        + current.irq.unwrap_or(0)
                        + current.softirq.unwrap_or(0)
                        + current.steal.unwrap_or(0)
                        + current.guest.unwrap_or(0)
                        + current.guest_nice.unwrap_or(0);

                    let total_delta = curr_total.saturating_sub(prev_total);
                    let idle_delta = curr_idle.saturating_sub(prev_idle);

                    if total_delta > 0 {
                        let usage = (1.0 - (idle_delta as f64 / total_delta as f64)) * 100.0;
                        usages.push(usage.round() as i8);
                    } else {
                        usages.push(0);
                    }
                }

                result = Some(CpuUsage {
                    timestamp: now_in_secs,
                    usages,
                });
            }
        }

        self.prev_cpu_times = Some(current_cpu_times);
        Ok(result)
    }
}
