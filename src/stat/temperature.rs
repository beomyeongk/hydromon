use crate::config::TemperatureConfig;
use crate::db::{NameMapper, Temperature};
use crate::schema::{DynamicSchema, SchemaField};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct TemperatureStats {
    // Sorted list of devices and their sensors
    devices: Vec<(String, Vec<(String, PathBuf)>)>,
    pub schema: Option<DynamicSchema>,
    pub schema_id: Option<i64>,
}

impl TemperatureStats {
    pub fn new(config: &TemperatureConfig) -> Self {
        let mut device_map = HashMap::new();

        if !config.enabled {
            return Self {
                devices: Vec::new(),
                schema: None,
                schema_id: None,
            };
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

        let mut sorted_devices: Vec<_> = device_map.into_iter().collect();
        sorted_devices.sort_by(|a, b| a.0.cmp(&b.0));

        let mut fields = Vec::new();
        for (dev_name, sensors) in &sorted_devices {
            for (label, _) in sensors {
                let name = format!("{}:{}", dev_name, label);
                fields.push(SchemaField {
                    name,
                    size: 2,
                    field_type: "int".to_string(),
                });
            }
        }

        let (schema, schema_id) = if !fields.is_empty() {
            let s = DynamicSchema::new(fields);
            let id = s.schema_id();
            (Some(s), Some(id))
        } else {
            (None, None)
        };

        Self {
            devices: sorted_devices,
            schema,
            schema_id,
        }
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



    pub fn collect(
        &self,
        timestamp: i64,
        _name_mapper: &NameMapper,
    ) -> Result<Option<Temperature>, Box<dyn std::error::Error>> {
        if self.schema_id.is_none() || self.devices.is_empty() {
            return Ok(None);
        }

        let mut data_bytes = Vec::new();
        let mut any_collected = false;

        for (_device_name, sensors) in &self.devices {
            for (_sensor_label, input_path) in sensors {
                let mut temp: i16 = 0; // Default to 0 if failed to read
                if let Ok(content) = fs::read_to_string(input_path) {
                    if let Ok(millidegrees) = content.trim().parse::<f64>() {
                        temp = (millidegrees / 100.0).round() as i16;
                        any_collected = true;
                    }
                }
                data_bytes.extend_from_slice(&temp.to_le_bytes());
            }
        }

        if !any_collected {
            return Ok(None);
        }

        Ok(Some(Temperature {
            timestamp,
            schema_id: self.schema_id.unwrap(),
            data: data_bytes,
        }))
    }
}
