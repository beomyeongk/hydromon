use std::io;
use procfs::CurrentSI;
use crate::db::CpuModes;

pub struct CpuModesStats {
    first_value_exists: bool,
    prev_user: u64,
    prev_nice: u64,
    prev_system: u64,
    prev_idle: u64,
    prev_iowait: u64,
    prev_irq: u64,
    prev_softirq: u64,
    prev_steal: u64,
    prev_guest: u64,
    prev_guest_nice: u64,
}

impl CpuModesStats {
    pub fn new() -> Self {
        CpuModesStats {
            first_value_exists: false,
            prev_user: 0,
            prev_nice: 0,
            prev_system: 0,
            prev_idle: 0,
            prev_iowait: 0,
            prev_irq: 0,
            prev_softirq: 0,
            prev_steal: 0,
            prev_guest: 0,
            prev_guest_nice: 0,
        }
    }

    pub fn update(&mut self, now_in_secs: i64) -> io::Result<Option<CpuModes>> {
        let stats = procfs::KernelStats::current()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let cpu = stats.total;

        let user = cpu.user;
        let nice = cpu.nice;
        let system = cpu.system;
        let idle = cpu.idle;
        let iowait = cpu.iowait.unwrap_or(0);
        let irq = cpu.irq.unwrap_or(0);
        let softirq = cpu.softirq.unwrap_or(0);
        let steal = cpu.steal.unwrap_or(0);
        let guest = cpu.guest.unwrap_or(0);
        let guest_nice = cpu.guest_nice.unwrap_or(0);

        let mut modes = None;

        if self.first_value_exists {
            let user_delta = user - self.prev_user;
            let nice_delta = nice - self.prev_nice;
            let system_delta = system - self.prev_system;
            let idle_delta = idle - self.prev_idle;
            let iowait_delta = iowait - self.prev_iowait;
            let irq_delta = irq - self.prev_irq;
            let softirq_delta = softirq - self.prev_softirq;
            let steal_delta = steal - self.prev_steal;
            let guest_delta = guest - self.prev_guest;
            let guest_nice_delta = guest_nice - self.prev_guest_nice;

            let total_delta = user_delta
                + nice_delta
                + system_delta
                + idle_delta
                + iowait_delta
                + irq_delta
                + softirq_delta
                + steal_delta
                + guest_delta
                + guest_nice_delta;

            if total_delta > 0 {
                modes = Some(CpuModes {
                    timestamp: now_in_secs,
                    user: ((user_delta as f64 / total_delta as f64) * 100.0) as i8,
                    nice: ((nice_delta as f64 / total_delta as f64) * 100.0) as i8,
                    system: ((system_delta as f64 / total_delta as f64) * 100.0) as i8,
                    idle: ((idle_delta as f64 / total_delta as f64) * 100.0) as i8,
                    iowait: ((iowait_delta as f64 / total_delta as f64) * 100.0) as i8,
                    irq: ((irq_delta as f64 / total_delta as f64) * 100.0) as i8,
                    softirq: ((softirq_delta as f64 / total_delta as f64) * 100.0) as i8,
                    steal: ((steal_delta as f64 / total_delta as f64) * 100.0) as i8,
                    guest: ((guest_delta as f64 / total_delta as f64) * 100.0) as i8,
                    guest_nice: ((guest_nice_delta as f64 / total_delta as f64) * 100.0) as i8,
                });
            }
        }

        self.first_value_exists = true;
        self.prev_user = user;
        self.prev_nice = nice;
        self.prev_system = system;
        self.prev_idle = idle;
        self.prev_iowait = iowait;
        self.prev_irq = irq;
        self.prev_softirq = softirq;
        self.prev_steal = steal;
        self.prev_guest = guest;
        self.prev_guest_nice = guest_nice;

        Ok(modes)
    }
}
