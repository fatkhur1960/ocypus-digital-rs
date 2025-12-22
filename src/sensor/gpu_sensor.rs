use crate::error::{OcypusError, Result};
use std::process::Command;

/// GPU temperature sensor
pub struct GpuSensor;

impl GpuSensor {
    /// Get the GPU temperature using the best available method
    pub fn get_temperature() -> Result<f32> {
        // Try different GPU monitoring tools in order of preference
        Self::try_nvidia_smi()
            .or_else(|_| Self::try_amd_smi())
            .or_else(|_| Self::try_rocm_smi())
            .or_else(|_| Self::try_sensors())
    }

    /// Try NVIDIA GPU temperature
    fn try_nvidia_smi() -> Result<f32> {
        let output = Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=temperature.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .map_err(|_| OcypusError::Sensor("nvidia-smi not available".to_string()))?;

        if !output.status.success() {
            return Err(OcypusError::Sensor("nvidia-smi command failed".to_string()));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let temp_str = text
            .lines()
            .next()
            .ok_or_else(|| OcypusError::Sensor("No output from nvidia-smi".to_string()))?
            .trim();

        temp_str.parse::<f32>().map_err(|e| {
            OcypusError::TemperatureParse(format!(
                "Failed to parse nvidia-smi temperature '{}': {}",
                temp_str, e
            ))
        })
    }

    /// Try AMD GPU temperature (new ROCm)
    fn try_amd_smi() -> Result<f32> {
        let output = Command::new("amd-smi")
            .args(&["metric", "--temperature"])
            .output()
            .map_err(|_| OcypusError::Sensor("amd-smi not available".to_string()))?;

        if !output.status.success() {
            return Err(OcypusError::Sensor("amd-smi command failed".to_string()));
        }

        let text = String::from_utf8_lossy(&output.stdout);

        for line in text.lines() {
            if line.to_lowercase().contains("edge") {
                if let Some(temp_str) = Self::extract_number(line) {
                    return temp_str.parse::<f32>().map_err(|e| {
                        OcypusError::TemperatureParse(format!(
                            "Failed to parse amd-smi temperature '{}': {}",
                            temp_str, e
                        ))
                    });
                }
            }
        }

        Err(OcypusError::Sensor(
            "No edge temperature found in amd-smi output".to_string(),
        ))
    }

    /// Try AMD GPU temperature (old ROCm)
    fn try_rocm_smi() -> Result<f32> {
        let output = Command::new("rocm-smi")
            .args(&["--showtemp"])
            .output()
            .map_err(|_| OcypusError::Sensor("rocm-smi not available".to_string()))?;

        if !output.status.success() {
            return Err(OcypusError::Sensor("rocm-smi command failed".to_string()));
        }

        let text = String::from_utf8_lossy(&output.stdout);

        for line in text.lines() {
            if let Some(temp_str) = Self::extract_number(line) {
                return temp_str.parse::<f32>().map_err(|e| {
                    OcypusError::TemperatureParse(format!(
                        "Failed to parse rocm-smi temperature '{}': {}",
                        temp_str, e
                    ))
                });
            }
        }

        Err(OcypusError::Sensor(
            "No temperature found in rocm-smi output".to_string(),
        ))
    }

    /// Try lm-sensors as fallback
    fn try_sensors() -> Result<f32> {
        let output = Command::new("sensors")
            .output()
            .map_err(|_| OcypusError::Sensor("sensors command not available".to_string()))?;

        if !output.status.success() {
            return Err(OcypusError::Sensor("sensors command failed".to_string()));
        }

        let text = String::from_utf8_lossy(&output.stdout);

        for line in text.lines() {
            let lower_line = line.to_lowercase();
            if lower_line.contains("gpu")
                || lower_line.contains("edge")
                || lower_line.contains("junction")
            {
                if let Some(temp_str) = Self::extract_number(line) {
                    return temp_str.parse::<f32>().map_err(|e| {
                        OcypusError::TemperatureParse(format!(
                            "Failed to parse sensors temperature '{}': {}",
                            temp_str, e
                        ))
                    });
                }
            }
        }

        Err(OcypusError::Sensor(
            "No GPU temperature found in sensors output".to_string(),
        ))
    }

    /// Extract the first floating-point number from a string
    fn extract_number(s: &str) -> Option<String> {
        let mut buf = String::new();
        let mut seen_digit = false;

        for c in s.chars() {
            if c.is_ascii_digit() || (c == '.' && seen_digit) {
                buf.push(c);
                seen_digit = true;
            } else if seen_digit {
                break;
            }
        }

        if buf.is_empty() {
            None
        } else {
            Some(buf)
        }
    }

    /// Check if any GPU sensor is available
    #[allow(unused)]
    pub fn is_available() -> bool {
        Self::try_nvidia_smi().is_ok()
            || Self::try_amd_smi().is_ok()
            || Self::try_rocm_smi().is_ok()
            || Self::try_sensors().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_sensor_availability() {
        let available = GpuSensor::is_available();
        assert!(available, "GPU sensor not available");
    }

    #[test]
    fn test_get_gpu_temperature() {
        if GpuSensor::is_available() {
            let temp = GpuSensor::get_temperature();
            assert!(temp.is_ok(), "Failed to get GPU temperature: {:?}", temp);

            if let Ok(temp) = temp {
                assert!(temp > 0.0, "Temperature should be positive: {}", temp);
                assert!(temp < 150.0, "Temperature seems too high: {}", temp);
            }
        }
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(
            GpuSensor::extract_number("Temperature: 45.5°C"),
            Some("45.5".to_string())
        );
        assert_eq!(GpuSensor::extract_number("45°C"), Some("45".to_string()));
        assert_eq!(GpuSensor::extract_number("No numbers here"), None);
        assert_eq!(GpuSensor::extract_number(""), None);
    }
}
