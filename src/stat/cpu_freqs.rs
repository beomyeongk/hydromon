use crate::db::CpuFreqs;
use procfs::{CpuInfo, Current};
use std::io;

pub struct CpuFreqsStats;

impl CpuFreqsStats {
    pub fn new() -> Self {
        CpuFreqsStats
    }

    pub fn collect(&self, now_in_secs: i64) -> io::Result<CpuFreqs> {
        let cpuinfo = CpuInfo::current().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut freqs_list = Vec::new();

        for cpu in cpuinfo.cpus {
            if let Some(mhz_str) = cpu.get("cpu MHz") {
                let mhz: f64 = mhz_str.parse().unwrap_or(0.0);
                // 1 unit = 100MHz
                freqs_list.push((mhz / 100.0) as i8);
            }
        }

        Ok(CpuFreqs {
            timestamp: now_in_secs,
            freqs: freqs_list,
        })
    }
}
