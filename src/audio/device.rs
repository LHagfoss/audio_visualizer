use cpal::Device;
use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;

pub struct AudioDeviceManager {
    host: cpal::Host,
}

impl AudioDeviceManager {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    pub fn list_devices(&self) -> Result<Vec<(usize, String)>, String> {
        let devices = self
            .host
            .devices()
            .map_err(|e| format!("Failed to fetch devices: {}", e))?;

        let mut device_list = Vec::new();
        for (index, device) in devices.enumerate() {
            let name = device
                .id()
                .map(|id| id.to_string())
                .unwrap_or_else(|_| "Unknown Device".to_string());
            device_list.push((index, name));
        }

        Ok(device_list)
    }

    pub fn get_device_by_index(&self, index: usize) -> Result<Device, String> {
        let mut devices = self
            .host
            .devices()
            .map_err(|e| format!("Failed to fetch devices: {}", e))?;

        devices
            .nth(index)
            .ok_or_else(|| format!("No device found at index {}", index))
    }
}
