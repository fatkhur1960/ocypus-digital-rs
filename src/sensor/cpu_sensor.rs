use crate::error::{OcypusError, Result};
use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;

/// CPU temperature sensor
pub struct CpuSensor;

impl CpuSensor {
    /// Get the CPU temperature using the best available method.
    ///
    /// Tries `sensors` first (most detailed), then falls back to
    /// kernel sysfs (`/sys/class/hwmon`, `/sys/class/thermal`) which
    /// needs no external binary.
    pub fn get_temperature() -> Result<f32> {
        Self::try_sensors()
            .or_else(|_| Self::try_hwmon())
            .or_else(|_| Self::try_thermal_zone())
    }

    /// Try lm-sensors `sensors` command
    fn try_sensors() -> Result<f32> {
        let output = Command::new("sensors").output().map_err(|e| {
            OcypusError::Sensor(format!("Failed to execute sensors command: {}", e))
        })?;

        if !output.status.success() {
            return Err(OcypusError::Sensor(
                "sensors command returned non-zero exit status".to_string(),
            ));
        }

        Self::parse_sensors_output(&String::from_utf8_lossy(&output.stdout))
    }

    /// Parse `sensors` stdout, extracted for unit testing
    fn parse_sensors_output(text: &str) -> Result<f32> {
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

            if let Some(captures) = re.captures(text) {
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

    /// Try `/sys/class/hwmon` (coretemp / k10temp / acpitz, no external deps)
    fn try_hwmon() -> Result<f32> {
        Self::read_hwmon_temp(Path::new("/sys/class/hwmon"))
    }

    /// Scan a hwmon base dir, extracted for unit testing with temp dirs
    fn read_hwmon_temp(base: &Path) -> Result<f32> {
        let entries = fs::read_dir(base).map_err(|e| {
            OcypusError::Sensor(format!("Failed to read {}: {}", base.display(), e))
        })?;

        let mut package_temp: Option<f32> = None;
        let mut core_max: Option<f32> = None;
        let mut any_cpu_temp: Option<f32> = None;

        for entry in entries.flatten() {
            let hwmon_dir = entry.path();
            let name = fs::read_to_string(hwmon_dir.join("name"))
                .unwrap_or_default()
                .trim()
                .to_lowercase();

            // Skip devices that are never the CPU
            if name.starts_with("nvme")
                || name.starts_with("iwlwifi")
                || name.starts_with("r8169")
                || name.starts_with("hidpp")
                || name.starts_with("battery")
                || name.starts_with("amdgpu")
                || name.starts_with("nouveau")
                || name.starts_with("nvidia")
            {
                continue;
            }

            // Only consider CPU-ish drivers here; k10temp/zenpower/coretemp/acpitz/soc
            let is_cpu_driver = name.contains("coretemp")
                || name.contains("k10temp")
                || name.contains("zenpower")
                || name.contains("acpitz")
                || name.contains("soc_thermal")
                || name.contains("cpu_thermal")
                || name.contains("x86_pkg_temp");
            if !is_cpu_driver {
                continue;
            }

            // Iterate temp{N}_input files
            let dir_entries = match fs::read_dir(&hwmon_dir) {
                Ok(d) => d,
                Err(_) => continue,
            };
            for f in dir_entries.flatten() {
                let fname = f.file_name().to_string_lossy().into_owned();
                if !fname.starts_with("temp") || !fname.ends_with("_input") {
                    continue;
                }
                let prefix = fname.trim_end_matches("_input");
                let raw = match fs::read_to_string(f.path()) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let millideg: f32 = match raw.trim().parse() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                // sanity: millidegree range -40°C..150°C
                if !(-40000.0..=150000.0).contains(&millideg) {
                    continue;
                }
                let temp = millideg / 1000.0;
                let label = fs::read_to_string(hwmon_dir.join(format!("{}_label", prefix)))
                    .unwrap_or_default()
                    .to_lowercase();

                if label.contains("package") || label.contains("tdie") || label.contains("tctl") {
                    // Prefer Package/Tdie over individual cores
                    match package_temp {
                        Some(cur) if temp <= cur => {}
                        _ => package_temp = Some(temp),
                    }
                } else if label.contains("core") {
                    core_max = Some(core_max.map_or(temp, |cur: f32| cur.max(temp)));
                } else {
                    any_cpu_temp = Some(any_cpu_temp.map_or(temp, |cur: f32| cur.max(temp)));
                }
            }
        }

        package_temp
            .or(core_max)
            .or(any_cpu_temp)
            .ok_or_else(|| {
                OcypusError::Sensor(
                    "No CPU temperature found in /sys/class/hwmon".to_string(),
                )
            })
    }

    /// Try `/sys/class/thermal` thermal zones as last resort
    fn try_thermal_zone() -> Result<f32> {
        Self::read_thermal_zone_temp(Path::new("/sys/class/thermal"))
    }

    /// Scan a thermal base dir, extracted for unit testing with temp dirs
    fn read_thermal_zone_temp(base: &Path) -> Result<f32> {
        let entries = fs::read_dir(base).map_err(|e| {
            OcypusError::Sensor(format!("Failed to read {}: {}", base.display(), e))
        })?;

        let mut preferred: Option<f32> = None;
        let mut fallback: Option<f32> = None;

        for entry in entries.flatten() {
            let zone = entry.path();
            let zone_name = zone.file_name().unwrap_or_default().to_string_lossy();
            if !zone_name.starts_with("thermal_zone") {
                continue;
            }
            let kind = fs::read_to_string(zone.join("type"))
                .unwrap_or_default()
                .trim()
                .to_lowercase();
            let raw = match fs::read_to_string(zone.join("temp")) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let millideg: f32 = match raw.trim().parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if !(-40000.0..=150000.0).contains(&millideg) {
                continue;
            }
            let temp = millideg / 1000.0;

            if kind.contains("x86_pkg_temp")
                || kind.contains("acpitz")
                || kind.contains("soc_thermal")
                || kind.contains("cpu-thermal")
                || kind.contains("k10temp")
            {
                preferred = Some(preferred.map_or(temp, |cur: f32| cur.max(temp)));
            } else if kind.contains("coretemp") || kind.contains("cpu") || kind.contains("pkg") {
                fallback = Some(fallback.map_or(temp, |cur: f32| cur.max(temp)));
            }
            // NOTE: iwlwifi / nvme / battery zones intentionally ignored
        }

        preferred.or(fallback).ok_or_else(|| {
            OcypusError::Sensor(
                "No CPU temperature found in /sys/class/thermal".to_string(),
            )
        })
    }

    /// Check if the sensor is available (any backend works)
    pub fn is_available() -> bool {
        Self::get_temperature().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as test_fs;

    #[test]
    fn test_cpu_sensor_availability() {
        // Should pass via sensors OR sysfs fallback (no external dep required)
        let available = CpuSensor::is_available();
        assert!(available, "No CPU temperature source available (sensors + sysfs)");
    }

    #[test]
    fn test_get_cpu_temperature() {
        // This test will only pass if any backend is available
        if CpuSensor::is_available() {
            let temp = CpuSensor::get_temperature();
            assert!(temp.is_ok(), "Failed to get CPU temperature: {:?}", temp);

            if let Ok(temp) = temp {
                assert!(temp > -40.0, "Temperature implausibly low: {}", temp);
                assert!(temp < 150.0, "Temperature seems too high: {}", temp);
            }
        }
    }

    #[test]
    fn test_parse_sensors_intel() {
        let out = "coretemp-isa-0000\nPackage id 0:  +45.0°C  (high = +80.0°C)\nCore 0: +43.0°C\n";
        assert!((CpuSensor::parse_sensors_output(out).unwrap() - 45.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_sensors_amd() {
        let out = "k10temp-pci-00c3\nTdie:         +50.5°C\nTctl:         +50.5°C\n";
        assert!((CpuSensor::parse_sensors_output(out).unwrap() - 50.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_sensors_not_found() {
        assert!(CpuSensor::parse_sensors_output("no temps here").is_err());
    }

    #[test]
    fn test_read_hwmon_prefers_package() {
        let dir = tempfile::tempdir().unwrap();
        let hw = dir.path().join("hwmon0");
        test_fs::create_dir(&hw).unwrap();
        test_fs::write(hw.join("name"), "coretemp\n").unwrap();
        test_fs::write(hw.join("temp1_input"), "45000\n").unwrap();
        test_fs::write(hw.join("temp1_label"), "Package id 0\n").unwrap();
        test_fs::write(hw.join("temp2_input"), "40000\n").unwrap();
        test_fs::write(hw.join("temp2_label"), "Core 0\n").unwrap();
        // nvme device must be ignored
        let nv = dir.path().join("hwmon1");
        test_fs::create_dir(&nv).unwrap();
        test_fs::write(nv.join("name"), "nvme\n").unwrap();
        test_fs::write(nv.join("temp1_input"), "99999\n").unwrap();

        let t = CpuSensor::read_hwmon_temp(dir.path()).unwrap();
        assert!((t - 45.0).abs() < 0.01, "got {}", t);
    }

    #[test]
    fn test_read_hwmon_no_cpu() {
        let dir = tempfile::tempdir().unwrap();
        let nv = dir.path().join("hwmon0");
        test_fs::create_dir(&nv).unwrap();
        test_fs::write(nv.join("name"), "nvme\n").unwrap();
        test_fs::write(nv.join("temp1_input"), "35000\n").unwrap();
        assert!(CpuSensor::read_hwmon_temp(dir.path()).is_err());
    }

    #[test]
    fn test_read_thermal_zone_prefers_pkg() {
        let dir = tempfile::tempdir().unwrap();
        let z0 = dir.path().join("thermal_zone0");
        test_fs::create_dir(&z0).unwrap();
        test_fs::write(z0.join("type"), "iwlwifi_1\n").unwrap();
        test_fs::write(z0.join("temp"), "25000\n").unwrap();
        let z1 = dir.path().join("thermal_zone1");
        test_fs::create_dir(&z1).unwrap();
        test_fs::write(z1.join("type"), "x86_pkg_temp\n").unwrap();
        test_fs::write(z1.join("temp"), "52000\n").unwrap();

        let t = CpuSensor::read_thermal_zone_temp(dir.path()).unwrap();
        assert!((t - 52.0).abs() < 0.01, "got {}", t);
    }

    #[test]
    fn test_read_thermal_zone_ignores_wifi_only() {
        let dir = tempfile::tempdir().unwrap();
        let z0 = dir.path().join("thermal_zone0");
        test_fs::create_dir(&z0).unwrap();
        test_fs::write(z0.join("type"), "iwlwifi_1\n").unwrap();
        test_fs::write(z0.join("temp"), "25000\n").unwrap();
        assert!(CpuSensor::read_thermal_zone_temp(dir.path()).is_err());
    }
}
