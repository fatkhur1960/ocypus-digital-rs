use crate::error::{OcypusError, Result};
use regex::Regex;
use std::process::Command;

/// CPU temperature sensor
pub struct CpuSensor;

impl CpuSensor {
    /// Get the CPU temperature using the best available method
    pub fn get_temperature() -> Result<f32> {
        let output = Command::new("sensors").output().map_err(|e| {
            OcypusError::Sensor(format!("Failed to execute sensors command: {}", e))
        })?;

        if !output.status.success() {
            return Err(OcypusError::Sensor(
                "sensors command returned non-zero exit status".to_string(),
            ));
        }

        let text = String::from_utf8_lossy(&output.stdout);

        // Look for the temperature in the output with various patterns
        let patterns = [
            r"Package id 0:\s*\+([0-9]+(?:\.[0-9]+)?)°C", // Intel
            r"Tdie:\s*\+([0-9]+(?:\.[0-9]+)?)°C",         // AMD real die temp
            r"Tctl:\s*\+([0-9]+(?:\.[0-9]+)?)°C",         // AMD control temp
            r"temp1:\s*\+([0-9]+(?:\.[0-9]+)?)°C",        // fallback
        ];

        for pattern in patterns {
            let re = Regex::new(pattern).map_err(|e| {
                OcypusError::Sensor(format!(
                    "Failed to compile regex pattern '{}': {}",
                    pattern, e
                ))
            })?;

            if let Some(captures) = re.captures(&text) {
                let temp_str = captures
                    .get(1)
                    .ok_or_else(|| {
                        OcypusError::Sensor("Failed to capture temperature".to_string())
                    })?
                    .as_str();

                return temp_str.parse::<f32>().map_err(|e| {
                    OcypusError::TemperatureParse(format!(
                        "Failed to parse temperature '{}': {}",
                        temp_str, e
                    ))
                });
            }
        }

        Err(OcypusError::Sensor(
            "CPU temperature not found in sensors output".to_string(),
        ))
    }

    /// Check if the sensor is available
    pub fn is_available() -> bool {
        Command::new("sensors")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_sensor_availability() {
        // This test will pass if 'sensors' command is available, fail otherwise
        let available = CpuSensor::is_available();
        assert!(available, "sensors command not available");
    }

    #[test]
    fn test_get_cpu_temperature() {
        // This test will only pass if 'sensors' command is available and returns valid data
        if CpuSensor::is_available() {
            let temp = CpuSensor::get_temperature();
            assert!(temp.is_ok(), "Failed to get CPU temperature: {:?}", temp);

            if let Ok(temp) = temp {
                assert!(temp > 0.0, "Temperature should be positive: {}", temp);
                assert!(temp < 150.0, "Temperature seems too high: {}", temp);
            }
        }
    }
}
