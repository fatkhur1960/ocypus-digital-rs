use crate::config::{PID, REPORT_ID, REPORT_LENGTH, VID};
use crate::error::{OcypusError, Result};
use hidapi::HidApi;
use log::{debug, info, warn};

/// Device communication handler
pub struct DeviceManager {
    api: HidApi,
    device: Option<hidapi::HidDevice>,
}

impl DeviceManager {
    /// Create a new device manager
    pub fn new() -> Result<Self> {
        let api = HidApi::new().map_err(|e| OcypusError::HidApi(e.to_string()))?;
        Ok(Self { api, device: None })
    }

    /// Connect to the Ocypus device
    pub fn connect(&mut self) -> Result<()> {
        info!("Scanning for Ocypus Iota L24 device...");

        for device_info in self.api.device_list() {
            if device_info.vendor_id() == VID && device_info.product_id() == PID {
                let path = device_info.path().to_string_lossy().into_owned();
                debug!("Found device at: {}", path);

                match self.api.open_path(device_info.path()) {
                    Ok(dev) => {
                        info!("Connected to Ocypus Iota L24 at {}", path);
                        self.device = Some(dev);
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("Failed to open device at {}: {}", path, e);
                        continue;
                    }
                }
            }
        }

        Err(OcypusError::Device(
            "No Ocypus Iota L24 device found".to_string(),
        ))
    }

    /// Send temperature data to the device
    pub fn send_temperature(
        &mut self,
        temp_celsius: f32,
        unit: crate::config::TemperatureUnit,
    ) -> Result<()> {
        let device = self
            .device
            .as_mut()
            .ok_or_else(|| OcypusError::Device("Device not connected".to_string()))?;

        let report = build_temperature_report(temp_celsius, unit)?;

        match device.write(&report) {
            Ok(bytes_written) => {
                debug!("Sent {} bytes to device", bytes_written);
                if bytes_written != REPORT_LENGTH {
                    warn!(
                        "Expected to write {} bytes, but wrote {}",
                        REPORT_LENGTH, bytes_written
                    );
                }
                Ok(())
            }
            Err(e) => Err(OcypusError::Device(format!(
                "Failed to write to device: {}",
                e
            ))),
        }
    }

    /// Check if device is connected
    #[allow(unused)]
    pub fn is_connected(&self) -> bool {
        self.device.is_some()
    }

    /// Reconnect to the device
    pub fn reconnect(&mut self) -> Result<()> {
        info!("Attempting to reconnect to device...");
        self.device = None;
        self.connect()
    }
}

/// Build temperature report for the device
fn build_temperature_report(
    temp_celsius: f32,
    unit: crate::config::TemperatureUnit,
) -> Result<[u8; REPORT_LENGTH]> {
    let mut report = [0u8; REPORT_LENGTH];
    report[0] = REPORT_ID;

    // Convert temperature based on unit
    let display_temp = match unit {
        crate::config::TemperatureUnit::Celsius => temp_celsius,
        crate::config::TemperatureUnit::Fahrenheit => temp_celsius * 9.0 / 5.0 + 32.0,
    };

    // Clamp temperature to 0-999°C (or equivalent in F)
    let clamped_temp = display_temp.clamp(0.0, 999.0);
    let temp_int = clamped_temp as u32;

    // Extract digits (hundreds, tens, ones)
    report[3] = (temp_int / 100) as u8;
    report[4] = ((temp_int / 10) % 10) as u8;
    report[5] = (temp_int % 10) as u8;

    debug!(
        "Built report: {:.1}°{} -> {}{}{}",
        display_temp,
        unit.as_char(),
        report[3],
        report[4],
        report[5]
    );

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_temperature_report_celsius() {
        let report =
            build_temperature_report(25.5, crate::config::TemperatureUnit::Celsius).unwrap();
        assert_eq!(report[0], REPORT_ID);
        assert_eq!(report[3], 0); // hundreds
        assert_eq!(report[4], 2); // tens
        assert_eq!(report[5], 5); // ones
    }

    #[test]
    fn test_build_temperature_report_fahrenheit() {
        let report =
            build_temperature_report(25.0, crate::config::TemperatureUnit::Fahrenheit).unwrap();
        assert_eq!(report[0], REPORT_ID);
        assert_eq!(report[3], 0); // hundreds
        assert_eq!(report[4], 7); // tens (25°C = 77°F)
        assert_eq!(report[5], 7); // ones
    }

    #[test]
    fn test_build_temperature_report_clamping() {
        // Test negative temperature
        let report =
            build_temperature_report(-10.0, crate::config::TemperatureUnit::Celsius).unwrap();
        assert_eq!(report[3], 0);
        assert_eq!(report[4], 0);
        assert_eq!(report[5], 0);

        // Test high temperature
        let report =
            build_temperature_report(1500.0, crate::config::TemperatureUnit::Celsius).unwrap();
        assert_eq!(report[3], 9);
        assert_eq!(report[4], 9);
        assert_eq!(report[5], 9);
    }
}
