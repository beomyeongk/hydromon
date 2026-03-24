mod common;
mod db;
mod http;
mod stat;

use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use time::OffsetDateTime;

mod config;

use config::Config;
use db::DbManager;
use stat::cpu_freqs::CpuFreqsStats;
use stat::cpu_modes::CpuModesStats;
use stat::cpu_usage::CpuUsageStats;
use stat::disk_io::DiskIoStats;
use stat::disk_storage::DiskStorageStats;
use stat::gpu_nvidia::GpuNvidiaStats;
use stat::memory_usage::MemoryUsageStats;
use stat::network_connection::NetworkConnectionStats;
use stat::network_traffic::NetworkTrafficStats;
use stat::sys_activity::SysActivityStats;
use stat::sys_summary::SysSummaryStats;
use stat::temperature::TemperatureStats;

fn main() -> Result<(), Box<dyn Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("\nShutting down program...");
    })?;

    let config = Config::initialize("hydromon.toml")?;
    println!("Loaded configuration: {:?}", config);

    let mut db_manager = DbManager::new("hydromon.db")?;
    println!("SQLite database loaded.");

    let mut cpu_modes_stats = CpuModesStats::new();
    let memory_usage_stats = MemoryUsageStats::new();
    let cpu_freqs_stats = CpuFreqsStats::new();
    let mut cpu_usage_stats = CpuUsageStats::new();
    let mut disk_io_stats = DiskIoStats::new();
    let disk_storage_stats = DiskStorageStats::new();
    let mut network_traffic_stats = NetworkTrafficStats::new();
    let mut network_connection_stats = NetworkConnectionStats::new();
    let sys_summary_stats = SysSummaryStats::new();
    let mut sys_activity_stats = SysActivityStats::new();
    let temperature_stats = TemperatureStats::new(&config.temperature);
    let mut gpu_nvidia_stats = GpuNvidiaStats::new();

    // Register all string names into name_map (INSERT OR IGNORE),
    // then load the entire table into a NameMapper for O(1) in-memory lookups.
    {
        let mut all_names: Vec<&str> = Vec::new();

        // disk_io device names
        for s in &config.disk_io.devices {
            all_names.push(s.as_str());
        }
        // disk_storage mount points
        for s in &config.disk_storage.mounts {
            all_names.push(s.as_str());
        }
        // network interfaces
        for s in &config.network_traffic.interfaces {
            all_names.push(s.as_str());
        }
        // gpu device names
        for s in &config.gpu_nvidia.devices {
            all_names.push(s.as_str());
        }

        // temperature: sensor names (discovered from filesystem at new())
        let temp_names = temperature_stats.all_names();

        // Merge owned strings as &str
        let all_names_owned: Vec<String> = all_names.iter().map(|s| s.to_string()).collect();
        let mut combined: Vec<&str> = all_names_owned.iter().map(|s| s.as_str()).collect();
        for s in &temp_names {
            combined.push(s.as_str());
        }

        db_manager.register_names(&combined)?;
    }

    let name_mapper = db_manager.load_name_mapper()?;
    println!("NameMapper loaded.");

    let http_handle = http::start("0.0.0.0:8080", "hydromon.db", running.clone());

    println!("Starting monitoring... (2s interval)");

    while running.load(Ordering::SeqCst) {
        let now = OffsetDateTime::now_utc();
        let now_in_secs = now.unix_timestamp();

        println!("Current time (UTC): {}", now_in_secs);

        let cpu_modes = cpu_modes_stats.update(now_in_secs)?;
        let mem_usage = memory_usage_stats.collect(now_in_secs)?;
        let cpu_freqs = cpu_freqs_stats.collect(now_in_secs)?;
        let cpu_usage = cpu_usage_stats.update(now_in_secs)?;
        let sys_summary = sys_summary_stats.collect(now_in_secs)?;
        let sys_activity = sys_activity_stats.update(now_in_secs)?;

        let disk_io = if config.disk_io.enabled {
            match disk_io_stats.update(now_in_secs, &config.disk_io.devices, &name_mapper) {
                Ok(stats) => Some(stats),
                Err(e) => {
                    eprintln!("Disk IO collection error: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let disk_storage = if config.disk_storage.enabled {
            match disk_storage_stats.update(now_in_secs, &config.disk_storage.mounts, &name_mapper)
            {
                Ok(stats) => Some(stats),
                Err(e) => {
                    eprintln!("Disk Storage collection error: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let network_traffic = if config.network_traffic.enabled {
            match network_traffic_stats.update(
                now_in_secs,
                &config.network_traffic.interfaces,
                &name_mapper,
            ) {
                Ok(stats) => Some(stats),
                Err(e) => {
                    eprintln!("Network Traffic collection error: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let network_connection = if config.network_connection.enabled {
            match network_connection_stats.update(now_in_secs) {
                Ok(stats) => Some(stats),
                Err(e) => {
                    eprintln!("Network Connection collection error: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let temperature = if config.temperature.enabled {
            match temperature_stats.collect(now_in_secs, &name_mapper) {
                Ok(stats) => stats,
                Err(e) => {
                    eprintln!("Temperature collection error: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let gpu_nvidia = if config.gpu_nvidia.enabled {
            match gpu_nvidia_stats.collect(now_in_secs, &config.gpu_nvidia, &name_mapper) {
                Ok(stats) => Some(stats),
                Err(e) => {
                    eprintln!("Gpu Nvidia collection error: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let tx = db_manager.transaction()?;

        if let Some(cpu_modes) = cpu_modes {
            DbManager::insert_cpu(&tx, &cpu_modes)?;
        }
        if let Some(cpu_usage) = cpu_usage {
            DbManager::insert_cpu_usage(&tx, &cpu_usage)?;
        }
        if let Some(disk_io_data) = disk_io {
            if !disk_io_data.is_empty() {
                DbManager::insert_disk_io(&tx, &disk_io_data)?;
            }
        }
        if let Some(disk_storage_data) = disk_storage {
            if !disk_storage_data.is_empty() {
                DbManager::insert_disk_storage(&tx, &disk_storage_data)?;
            }
        }
        if let Some(network_traffic_data) = network_traffic {
            if !network_traffic_data.is_empty() {
                DbManager::insert_network_traffic(&tx, &network_traffic_data)?;
            }
        }
        if let Some(network_connection_data) = network_connection {
            DbManager::insert_network_connection(&tx, &network_connection_data)?;
        }
        DbManager::insert_sys_summary(&tx, &sys_summary)?;
        if let Some(act) = sys_activity {
            DbManager::insert_sys_activity(&tx, &act)?;
        }
        DbManager::insert_memory(&tx, &mem_usage)?;
        DbManager::insert_cpu_freqs(&tx, &cpu_freqs)?;
        if let Some(temp_data) = temperature {
            DbManager::insert_temperature(&tx, &temp_data)?;
        }
        if let Some(gpu_data) = gpu_nvidia {
            if !gpu_data.is_empty() {
                DbManager::insert_gpu_nvidia(&tx, &gpu_data)?;
            }
        }

        tx.commit()?;

        println!("Data saved.");

        thread::sleep(Duration::from_secs(2));
    }

    db_manager.checkpoint()?;
    http_handle.join().ok();

    println!("Terminated gracefully.");
    Ok(())
}
