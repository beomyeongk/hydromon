use rusqlite::{Connection, Result, Transaction, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub struct DbManager {
    conn: Connection,
}

/// In-memory cache of name → id mappings from the `name_map` table.
/// Populated once at startup; no DB reads needed during the main loop.
pub struct NameMapper {
    map: HashMap<String, i64>,
}

impl NameMapper {
    /// Look up the integer ID for a given name.
    /// Panics if the name was not registered at startup (programming error).
    pub fn get(&self, name: &str) -> i64 {
        *self
            .map
            .get(name)
            .unwrap_or_else(|| panic!("NameMapper: name '{}' was not registered at startup", name))
    }
}

pub struct CpuModes {
    pub timestamp: i64,
    pub user: i8,
    pub nice: i8,
    pub system: i8,
    pub idle: i8,
    pub iowait: i8,
    pub irq: i8,
    pub softirq: i8,
    pub steal: i8,
    pub guest: i8,
    pub guest_nice: i8,
}

#[derive(Serialize, Deserialize)]
pub struct CpuFreqs {
    pub timestamp: i64,
    pub freqs: Vec<i8>, // 1 unit = 100MHz, index is core_id
}

pub struct CpuUsage {
    pub timestamp: i64,
    pub usages: Vec<i8>, // 1 unit = 1%, index is core_id
}

pub struct MemoryUsage {
    pub timestamp: i64,
    pub total: u32,
    pub free: u32,
    pub available: u32,
    pub buffers: u32,
    pub cached: u32,
    pub swap_total: u32,
    pub swap_usage: u32,
}

pub struct DiskIo {
    pub timestamp: i64,
    pub name_id: i64, // FK → name_map.id  (was: device_name TEXT)
    pub r_kbps: u32,
    pub w_kbps: u32,
    pub r_await: u32,
    pub w_await: u32,
    pub aqu_sz: u32,
    pub util: u32,
    pub iops: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DiskStorage {
    pub timestamp: i64,
    pub name_id: i64, // FK → name_map.id  (was: mount_point TEXT)
    pub total: u32,   // 1 unit = 16 KiB
    pub used: u32,    // 1 unit = 16 KiB
    pub num_inodes: i64,
}

pub struct NetworkTraffic {
    pub timestamp: i64,
    pub name_id: i64, // FK → name_map.id  (was: interface TEXT)
    pub rx_kbps: u32,
    pub tx_kbps: u32,
    pub rx_pckps: u32,
    pub tx_pckps: u32,
}

pub struct NetworkConnection {
    pub timestamp: i64,
    pub tcp_syn_sent: u32,
    pub tcp_syn_recv: u32,
    pub tcp_established: u32,
    pub tcp_time_wait: u32,
    pub tcp_close_wait: u32,
    pub tcp_listen: u32,
    pub tcp_closing: u32,
}

pub struct SysSummary {
    pub timestamp: i64,
    pub uptime: u32,
    pub total_tasks: u32,
    pub load_avg_1m: u32,
    pub num_fds: u32,
}

pub struct SysActivity {
    pub timestamp: i64,
    pub intr: u64,
    pub ctxt: u64,
}

pub struct SysTemp {
    pub timestamp: i64,
    pub device_id: i64, // FK → name_map.id  (was: device_name TEXT)
    pub sensor_id: i64, // FK → name_map.id  (was: sensor_label TEXT)
    pub temp: i32,
}

pub struct GpuNvidia {
    pub timestamp: i64,
    pub name_id: i64, // FK → name_map.id  (was: device_name TEXT)
    pub fan_speed: u32,
    pub temp: i32,
    pub power_w: u32,
    pub vram_used_mib: u32,
    pub vram_total_mib: u32,
}

impl DbManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;\n\
             PRAGMA synchronous = NORMAL;"
        )?;

        let db = DbManager { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        // Universal name → integer-id lookup table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS name_map (
                id   INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS cpu_modes (
                timestamp INTEGER PRIMARY KEY,
                user INTEGER,
                nice INTEGER,
                system INTEGER,
                idle INTEGER,
                iowait INTEGER,
                irq INTEGER,
                softirq INTEGER,
                steal INTEGER,
                guest INTEGER,
                guest_nice INTEGER
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS memory_usage (
                timestamp INTEGER PRIMARY KEY,
                total INTEGER,
                free INTEGER,
                available INTEGER,
                buffers INTEGER,
                cached INTEGER,
                swap_total INTEGER,
                swap_usage INTEGER
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS cpu_freqs (
                timestamp INTEGER PRIMARY KEY,
                freqs BLOB
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS cpu_usage (
                timestamp INTEGER PRIMARY KEY,
                usages BLOB
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS disk_io (
                timestamp INTEGER,
                name_id INTEGER,
                r_kbps INTEGER,
                w_kbps INTEGER,
                r_await INTEGER,
                w_await INTEGER,
                aqu_sz INTEGER,
                util INTEGER,
                iops INTEGER,
                PRIMARY KEY (timestamp, name_id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS disk_storage (
                timestamp INTEGER,
                name_id INTEGER,
                total INTEGER,
                used INTEGER,
                num_inodes INTEGER,
                PRIMARY KEY (timestamp, name_id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS network_traffic (
                timestamp INTEGER,
                name_id INTEGER,
                rx_kbps INTEGER,
                tx_kbps INTEGER,
                rx_pckps INTEGER,
                tx_pckps INTEGER,
                PRIMARY KEY (timestamp, name_id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS network_connection (
                timestamp INTEGER PRIMARY KEY,
                tcp_syn_sent INTEGER,
                tcp_syn_recv INTEGER,
                tcp_established INTEGER,
                tcp_time_wait INTEGER,
                tcp_close_wait INTEGER,
                tcp_listen INTEGER,
                tcp_closing INTEGER
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sys_summary (
                timestamp INTEGER PRIMARY KEY,
                uptime INTEGER,
                total_tasks INTEGER,
                load_avg_1m INTEGER,
                num_fds INTEGER
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sys_activity (
                timestamp INTEGER PRIMARY KEY,
                intr INTEGER,
                ctxt INTEGER
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sys_temp (
                timestamp INTEGER,
                device_id INTEGER,
                sensor_id INTEGER,
                temp INTEGER,
                PRIMARY KEY (timestamp, device_id, sensor_id)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS gpu_nvidia (
                timestamp INTEGER,
                name_id INTEGER,
                fan_speed INTEGER,
                temp INTEGER,
                power_w INTEGER,
                vram_used_mib INTEGER,
                vram_total_mib INTEGER,
                PRIMARY KEY (timestamp, name_id)
            )",
            [],
        )?;

        Ok(())
    }

    /// Insert names that are not yet in `name_map` (INSERT OR IGNORE).
    /// Call once at startup with every possible string key your metrics will use.
    pub fn register_names(&self, names: &[&str]) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare("INSERT OR IGNORE INTO name_map (name) VALUES (?1)")?;
        for name in names {
            stmt.execute([name])?;
        }
        Ok(())
    }

    /// Load the entire `name_map` table into a `NameMapper` for O(1) in-memory lookups.
    pub fn load_name_mapper(&self) -> Result<NameMapper> {
        let mut stmt = self.conn.prepare("SELECT id, name FROM name_map")?;
        let map: HashMap<String, i64> = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                Ok((name, id))
            })?
            .collect::<Result<_>>()?;
        Ok(NameMapper { map })
    }

    pub fn transaction(&mut self) -> Result<Transaction<'_>> {
        self.conn.transaction()
    }

    pub fn checkpoint(&self) -> Result<()> {
        self.conn.execute_batch("PRAGMA wal_checkpoint(FULL);")
    }


    pub fn insert_cpu(tx: &Transaction, metric: &CpuModes) -> Result<()> {
        tx.execute(
            "INSERT INTO cpu_modes (timestamp, user, nice, system, idle, iowait, irq, softirq, steal, guest, guest_nice)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                metric.timestamp,
                metric.user,
                metric.nice,
                metric.system,
                metric.idle,
                metric.iowait,
                metric.irq,
                metric.softirq,
                metric.steal,
                metric.guest,
                metric.guest_nice,
            ],
        )?;
        Ok(())
    }

    pub fn insert_memory(tx: &Transaction, metric: &MemoryUsage) -> Result<()> {
        tx.execute(
            "INSERT INTO memory_usage (timestamp, total, free, available, buffers, cached, swap_total, swap_usage)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                metric.timestamp,
                metric.total,
                metric.free,
                metric.available,
                metric.buffers,
                metric.cached,
                metric.swap_total,
                metric.swap_usage,
            ],
        )?;
        Ok(())
    }

    pub fn insert_cpu_freqs(tx: &Transaction, metric: &CpuFreqs) -> Result<()> {
        let json_data = serde_json::to_string(&metric.freqs)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        tx.execute(
            "INSERT INTO cpu_freqs (timestamp, freqs) VALUES (?1, jsonb(?2))",
            params![metric.timestamp, json_data],
        )?;
        Ok(())
    }

    pub fn insert_cpu_usage(tx: &Transaction, metric: &CpuUsage) -> Result<()> {
        let json_data = serde_json::to_string(&metric.usages)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        tx.execute(
            "INSERT INTO cpu_usage (timestamp, usages) VALUES (?1, jsonb(?2))",
            params![metric.timestamp, json_data],
        )?;
        Ok(())
    }

    pub fn insert_disk_io(tx: &Transaction, metrics: &[DiskIo]) -> Result<()> {
        let mut stmt = tx.prepare(
            "INSERT INTO disk_io (timestamp, name_id, r_kbps, w_kbps, r_await, w_await, aqu_sz, util, iops)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        )?;

        for metric in metrics {
            stmt.execute(params![
                metric.timestamp,
                metric.name_id,
                metric.r_kbps,
                metric.w_kbps,
                metric.r_await,
                metric.w_await,
                metric.aqu_sz,
                metric.util,
                metric.iops,
            ])?;
        }
        Ok(())
    }

    pub fn insert_disk_storage(
        tx: &Transaction,
        stats: &[DiskStorage],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = tx.prepare(
            "INSERT INTO disk_storage (timestamp, name_id, total, used, num_inodes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        for stat in stats {
            stmt.execute((
                stat.timestamp,
                stat.name_id,
                stat.total,
                stat.used,
                stat.num_inodes,
            ))?;
        }
        Ok(())
    }

    pub fn insert_network_traffic(tx: &Transaction, metrics: &[NetworkTraffic]) -> Result<()> {
        let mut stmt = tx.prepare(
            "INSERT INTO network_traffic (timestamp, name_id, rx_kbps, tx_kbps, rx_pckps, tx_pckps)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;

        for metric in metrics {
            stmt.execute(params![
                metric.timestamp,
                metric.name_id,
                metric.rx_kbps,
                metric.tx_kbps,
                metric.rx_pckps,
                metric.tx_pckps,
            ])?;
        }
        Ok(())
    }

    pub fn insert_network_connection(tx: &Transaction, metric: &NetworkConnection) -> Result<()> {
        tx.execute(
            "INSERT INTO network_connection (
                timestamp, tcp_syn_sent, tcp_syn_recv, tcp_established,
                tcp_time_wait, tcp_close_wait, tcp_listen, tcp_closing
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                metric.timestamp,
                metric.tcp_syn_sent,
                metric.tcp_syn_recv,
                metric.tcp_established,
                metric.tcp_time_wait,
                metric.tcp_close_wait,
                metric.tcp_listen,
                metric.tcp_closing,
            ],
        )?;
        Ok(())
    }

    pub fn insert_sys_summary(tx: &Transaction, metric: &SysSummary) -> Result<()> {
        tx.execute(
            "INSERT INTO sys_summary (
                timestamp, uptime, total_tasks, load_avg_1m, num_fds
            ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                metric.timestamp,
                metric.uptime,
                metric.total_tasks,
                metric.load_avg_1m,
                metric.num_fds,
            ],
        )?;
        Ok(())
    }

    pub fn insert_sys_activity(tx: &Transaction, metric: &SysActivity) -> Result<()> {
        tx.execute(
            "INSERT INTO sys_activity (
                timestamp, intr, ctxt
            ) VALUES (?1, ?2, ?3)",
            params![metric.timestamp, metric.intr, metric.ctxt,],
        )?;
        Ok(())
    }

    pub fn insert_sys_temp(tx: &Transaction, metrics: &[SysTemp]) -> Result<()> {
        let mut stmt = tx.prepare(
            "INSERT INTO sys_temp (timestamp, device_id, sensor_id, temp)
             VALUES (?1, ?2, ?3, ?4)",
        )?;

        for metric in metrics {
            stmt.execute(params![
                metric.timestamp,
                metric.device_id,
                metric.sensor_id,
                metric.temp,
            ])?;
        }
        Ok(())
    }

    pub fn insert_gpu_nvidia(tx: &Transaction, metrics: &[GpuNvidia]) -> Result<()> {
        let mut stmt = tx.prepare(
            "INSERT INTO gpu_nvidia (timestamp, name_id, fan_speed, temp, power_w, vram_used_mib, vram_total_mib)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;

        for metric in metrics {
            stmt.execute(params![
                metric.timestamp,
                metric.name_id,
                metric.fan_speed,
                metric.temp,
                metric.power_w,
                metric.vram_used_mib,
                metric.vram_total_mib,
            ])?;
        }
        Ok(())
    }

    // end of primary impl block
}

impl Drop for DbManager {
    fn drop(&mut self) {
        if let Err(e) = self.checkpoint() {
            eprintln!(
                "hydromon: failed to checkpoint WAL on shutdown: {}",
                e
            );
        }
    }
}
