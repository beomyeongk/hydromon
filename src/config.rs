use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub cpu_usage: CpuUsageConfig,
    pub cpu_freqs: CpuFreqsConfig,
    pub cpu_modes: CpuModesConfig,
    pub memory_usage: MemoryUsageConfig,
    pub disk_io: DiskIoConfig,
    pub disk_storage: DiskStorageConfig,
    pub network_traffic: NetworkTrafficConfig,
    pub network_connection: NetworkConnectionConfig,
    pub sys_temp: SysTempConfig,
    pub gpu_nvidia: GpuNvidiaConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CpuUsageConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CpuFreqsConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CpuModesConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemoryUsageConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiskIoConfig {
    pub enabled: bool,
    pub devices: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DiskStorageConfig {
    pub enabled: bool,
    pub mounts: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct NetworkTrafficConfig {
    pub enabled: bool,
    pub interfaces: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct NetworkConnectionConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SysTempConfig {
    pub enabled: bool,
    pub devices: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct GpuNvidiaConfig {
    pub enabled: bool,
    pub devices: Vec<String>,
}

impl Config {
    pub fn initialize<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Self::create_default_config();
            let content = toml::to_string(&config)?;
            let mut file = fs::File::create(path)?;
            file.write_all(content.as_bytes())?;
            println!("Configuration file created: {:?}", path);
            Ok(config)
        }
    }

    fn create_default_config() -> Self {
        Config {
            cpu_usage: CpuUsageConfig { enabled: true },
            cpu_freqs: CpuFreqsConfig { enabled: true },
            cpu_modes: CpuModesConfig { enabled: true },
            memory_usage: MemoryUsageConfig { enabled: true },
            disk_io: DiskIoConfig {
                enabled: true,
                devices: detect_block_devices(),
            },
            disk_storage: DiskStorageConfig {
                enabled: true,
                mounts: detect_mount_points(),
            },
            network_traffic: NetworkTrafficConfig {
                enabled: true,
                interfaces: detect_network_interfaces(),
            },
            network_connection: NetworkConnectionConfig { enabled: true },
            sys_temp: SysTempConfig {
                enabled: true,
                devices: detect_sys_temp_devices(),
            },
            gpu_nvidia: GpuNvidiaConfig {
                enabled: true,
                devices: detect_gpu_nvidia(),
            },
        }
    }
}

fn detect_block_devices() -> Vec<String> {
    let mut devices = Vec::new();
    // Simple disk detection using /sys/block
    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with("sd")
                    || name.starts_with("nvme")
                    || name.starts_with("vd")
                    || name.starts_with("xvd")
                {
                    devices.push(name);
                }
            }
        }
    }
    devices.sort();
    devices
}

fn detect_mount_points() -> Vec<String> {
    let mut mounts = Vec::new();
    if let Ok(content) = fs::read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let source = parts[0];
                let mount_point = parts[1];
                let fs_type = parts[2];

                if source.starts_with("/dev/")
                    && !mount_point.starts_with("/boot/efi")
                    && (fs_type == "ext4"
                        || fs_type == "xfs"
                        || fs_type == "btrfs"
                        || fs_type == "zfs"
                        || fs_type == "vfat"
                        || fs_type == "ntfs"
                        || fs_type == "apfs")
                {
                    mounts.push(mount_point.to_string());
                }
            }
        }
    }
    mounts.sort();
    mounts.dedup();
    mounts
}

fn detect_network_interfaces() -> Vec<String> {
    let mut interfaces = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name != "lo" {
                    interfaces.push(name);
                }
            }
        }
    }
    interfaces.sort();
    interfaces
}

fn detect_sys_temp_devices() -> Vec<String> {
    let mut devices = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.starts_with("hwmon") {
                    let name_path = entry.path().join("name");
                    if let Ok(name_content) = fs::read_to_string(name_path) {
                        let base_name = name_content.trim().to_string();
                        // Extract hwmon index from "hwmonN"
                        if let Some(index) = file_name.strip_prefix("hwmon") {
                            devices.push(format!("{}_{}", base_name, index));
                        }
                    }
                }
            }
        }
    }
    // E.g., nvme_2, coretemp_7, etc.
    devices.sort();
    devices
}

fn detect_gpu_nvidia() -> Vec<String> {
    let mut devices = Vec::new();
    if let Ok(nvml) = nvml_wrapper::Nvml::init() {
        if let Ok(count) = nvml.device_count() {
            for i in 0..count {
                devices.push(format!("gpu{}", i));
            }
        }
    }
    devices
}
