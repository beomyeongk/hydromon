use crate::config::TemperatureConfig;
use crate::db::{NameMapper, Temperature};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct TemperatureStats {
    // Maps a logical device name (e.g., nvme_0) to a list of (sensor_label, temp_input_path)
    device_map: HashMap<String, Vec<(String, PathBuf)>>,
}

impl TemperatureStats {
    pub fn new(config: &TemperatureConfig) -> Self {
        let mut device_map = HashMap::new();

        if !config.enabled {
            return Self { device_map };
        }

        // Build a temporary map of all available hardware monitors on the system
        let mut system_devices: HashMap<String, PathBuf> = HashMap::new();
        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with("hwmon") {
                        let path = entry.path();
                        let name_path = path.join("name");
                        if let Ok(name_content) = fs::read_to_string(&name_path) {
                            let base_name = name_content.trim().to_string();
                            if let Some(index) = file_name.strip_prefix("hwmon") {
                                let logical_name = format!("{}_{}", base_name, index);
                                system_devices.insert(logical_name, path);
                            }
                        }
                    }
                }
            }
        }

        // Only map the devices specified in the config
        for device_name in &config.devices {
            if let Some(base_path) = system_devices.get(device_name) {
                let allowed = config.sensor_filters.get(device_name);
                let sensors = Self::discover_sensors(base_path, allowed);
                if !sensors.is_empty() {
                    device_map.insert(device_name.clone(), sensors);
                }
            }
        }

        Self { device_map }
    }

    fn discover_sensors(
        base_path: &Path,
        allowed_labels: Option<&Vec<String>>,
    ) -> Vec<(String, PathBuf)> {
        let mut sensors = Vec::new();
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    // Look for temp*_input files
                    if file_name.starts_with("temp") && file_name.ends_with("_input") {
                        let input_path = entry.path();

                        // Extract the prefix, e.g., "temp1" from "temp1_input"
                        let prefix = file_name.trim_end_matches("_input");
                        let label_path = base_path.join(format!("{}_label", prefix));

                        let label = if let Ok(label_content) = fs::read_to_string(&label_path) {
                            label_content.trim().to_string()
                        } else {
                            // Fallback to the temp prefix if label doesn't exist
                            prefix.to_string()
                        };

                        // If a filter is specified, only collect the labels in the filter
                        if let Some(filter) = allowed_labels {
                            if !filter.contains(&label) {
                                continue;
                            }
                        }

                        sensors.push((label, input_path));
                    }
                }
            }
        }
        sensors.sort_by(|a, b| a.0.cmp(&b.0));
        sensors
    }

    // Returns all (device_name, sensor_label) pairs discovered during construction.
    // Used at startup to register these names into the `name_map` table.
    pub fn all_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for (device_name, sensors) in &self.device_map {
            for (sensor_label, _) in sensors {
                names.push(format!("{}:{}", device_name, sensor_label));
            }
        }
        names
    }

    pub fn collect(
        &self,
        timestamp: i64,
        name_mapper: &NameMapper,
    ) -> Result<Option<Temperature>, Box<dyn std::error::Error>> {
        let mut sensor_data: HashMap<String, i32> = HashMap::new();

        for (device_name, sensors) in &self.device_map {
            for (sensor_label, input_path) in sensors {
                if let Ok(content) = fs::read_to_string(input_path) {
                    if let Ok(millidegrees) = content.trim().parse::<f64>() {
                        let temp = (millidegrees / 1000.0).round() as i32;
                        
                        let full_name = format!("{}:{}", device_name, sensor_label);
                        let id = name_mapper.get(&full_name);
                        sensor_data.insert(id.to_string(), temp);
                    }
                }
            }
        }

        if sensor_data.is_empty() {
            return Ok(None);
        }

        let json_data = serde_json::to_string(&sensor_data)?;

        Ok(Some(Temperature {
            timestamp,
            data: json_data,
        }))
    }
}
