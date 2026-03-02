use crate::db::SysActivity;
use std::fs::File;
use std::io::{self, Read};

pub struct SysActivityStats {
    first_value_exists: bool,
    prev_timestamp: std::time::Instant,
    prev_intr: u64,
    prev_ctxt: u64,
    buffer: String,
}

impl SysActivityStats {
    pub fn new() -> Self {
        SysActivityStats {
            first_value_exists: false,
            prev_timestamp: std::time::Instant::now(),
            prev_intr: 0,
            prev_ctxt: 0,
            buffer: String::with_capacity(8192),
        }
    }

    pub fn update(&mut self, now_in_secs: i64) -> io::Result<Option<SysActivity>> {
        self.buffer.clear();
        let mut file = File::open("/proc/stat")?;
        file.read_to_string(&mut self.buffer)?;

        let mut intr: Option<u64> = None;
        let mut ctxt: Option<u64> = None;

        for line in self.buffer.lines() {
            if let Some(rest) = line.strip_prefix("intr ") {
                if let Some(val_str) = rest.split_whitespace().next() {
                    intr = val_str.parse().ok();
                }
            } else if let Some(rest) = line.strip_prefix("ctxt ") {
                if let Some(val_str) = rest.split_whitespace().next() {
                    ctxt = val_str.parse().ok();
                }
                break;
            }
        }

        let now = std::time::Instant::now();
        let intr_val = intr.unwrap_or(0);
        let ctxt_val = ctxt.unwrap_or(0);

        let mut activity = None;

        if self.first_value_exists {
            let time_delta = now.duration_since(self.prev_timestamp).as_secs_f64();

            if time_delta > 0.0 {
                let intr_delta = intr_val.saturating_sub(self.prev_intr);
                let ctxt_delta = ctxt_val.saturating_sub(self.prev_ctxt);

                let intr_hz = (intr_delta as f64 / time_delta) as u64;
                let ctxt_hz = (ctxt_delta as f64 / time_delta) as u64;

                activity = Some(SysActivity {
                    timestamp: now_in_secs,
                    intr: intr_hz,
                    ctxt: ctxt_hz,
                });
            }
        }

        self.first_value_exists = true;
        self.prev_timestamp = now;
        self.prev_intr = intr_val;
        self.prev_ctxt = ctxt_val;

        Ok(activity)
    }
}
