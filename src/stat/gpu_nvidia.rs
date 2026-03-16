use crate::config::GpuNvidiaConfig;
use crate::db::{GpuNvidia, NameMapper};
use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};

pub struct GpuNvidiaStats {
    nvml: Option<Nvml>,
}

impl GpuNvidiaStats {
    pub fn new() -> Self {
        // Initialize NVML, ignoring errors if it fails (e.g., driver missing)
        let nvml = Nvml::init().ok();
        Self { nvml }
    }

    pub fn collect(
        &mut self,
        timestamp: i64,
        config: &GpuNvidiaConfig,
        name_mapper: &NameMapper,
    ) -> Result<Vec<GpuNvidia>, String> {
        let mut metrics = Vec::new();

        if config.devices.is_empty() {
            return Ok(metrics);
        }

        if let Some(nvml) = &self.nvml {
            for (i, device_name) in config.devices.iter().enumerate() {
                if let Ok(device) = nvml.device_by_index(i as u32) {
                    let fan_speed = device.fan_speed(0).unwrap_or(0);
                    let temp = device.temperature(TemperatureSensor::Gpu).unwrap_or(0) as i32;
                    let power_w = device.power_usage().map(|mw| mw / 1000).unwrap_or(0);
                    let vram_used_mib = device
                        .memory_info()
                        .map(|info| (info.used / (1024 * 1024)) as u32)
                        .unwrap_or(0);
                    let vram_total_mib = device
                        .memory_info()
                        .map(|info| (info.total / (1024 * 1024)) as u32)
                        .unwrap_or(0);

                    let gpu_clock_mhz = device.clock_info(Clock::Graphics).unwrap_or(0);
                    let mem_clock_mhz = device.clock_info(Clock::Memory).unwrap_or(0);
                    let gpu_util = device
                        .utilization_rates()
                        .map(|u| u.gpu)
                        .unwrap_or(0);
                    let enc_util = device
                        .encoder_utilization()
                        .map(|u| u.utilization)
                        .unwrap_or(0);
                    let dec_util = device
                        .decoder_utilization()
                        .map(|u| u.utilization)
                        .unwrap_or(0);

                    metrics.push(GpuNvidia {
                        timestamp,
                        name_id: name_mapper.get(device_name),
                        fan_speed,
                        temp,
                        power_w,
                        vram_used_mib,
                        vram_total_mib,
                        gpu_clock_mhz,
                        mem_clock_mhz,
                        gpu_util,
                        enc_util,
                        dec_util,
                    });
                }
            }
        }
        Ok(metrics)
    }
}
