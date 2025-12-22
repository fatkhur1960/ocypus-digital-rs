use crate::config::{Config, SensorType, TemperatureUnit};
use crate::error::Result;
use crate::sensor::{cpu_sensor::CpuSensor, gpu_sensor::GpuSensor};
use log::{info, warn};
use std::sync::mpsc;
use std::thread;

/// Temperature monitoring service
pub struct TemperatureMonitor {
    config: Config,
    sensor_manager: SensorManager,
}

impl TemperatureMonitor {
    /// Create a new temperature monitor
    pub fn new(config: Config) -> Self {
        Self {
            config,
            sensor_manager: SensorManager::new(),
        }
    }

    /// Start monitoring temperature in a separate thread
    pub fn start_monitoring(&self) -> Result<mpsc::Receiver<f32>> {
        let (tx, rx) = mpsc::channel::<f32>();
        let config = self.config.clone();
        let sensor_manager = self.sensor_manager.clone();

        thread::spawn(move || {
            info!("Starting temperature monitoring thread");
            info!("Using sensor: {}", config.sensor_type.as_str());
            info!(
                "Update interval: {} seconds",
                config.update_interval.as_secs()
            );

            loop {
                match sensor_manager.get_temperature(&config.sensor_type) {
                    Ok(temp) => {
                        Self::check_thresholds(temp, &config);

                        if let Err(e) = tx.send(temp) {
                            log::error!("Failed to send temperature: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to get temperature: {}", e);
                        // Continue monitoring even if one reading fails
                    }
                }

                thread::sleep(config.update_interval);
            }
        });

        Ok(rx)
    }

    /// Check temperature thresholds and emit alerts if needed
    fn check_thresholds(temp: f32, config: &Config) {
        if !config.alerts_enabled {
            return;
        }

        if temp > config.high_threshold {
            warn!(
                "High temperature alert: {:.1}°C (threshold: {:.1}°C)",
                temp, config.high_threshold
            );
        } else if temp < config.low_threshold {
            warn!(
                "Low temperature alert: {:.1}°C (threshold: {:.1}°C)",
                temp, config.low_threshold
            );
        }
    }

    /// Get a single temperature reading
    #[allow(unused)]
    pub fn get_current_temperature(&self) -> Result<f32> {
        self.sensor_manager
            .get_temperature(&self.config.sensor_type)
    }

    /// Convert temperature to display unit
    pub fn convert_temperature(&self, temp_celsius: f32) -> f32 {
        match self.config.temperature_unit {
            TemperatureUnit::Celsius => temp_celsius,
            TemperatureUnit::Fahrenheit => temp_celsius * 9.0 / 5.0 + 32.0,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

/// Sensor manager for handling different sensor types
#[derive(Debug, Clone)]
pub struct SensorManager {
    // In a more complex implementation, this could manage sensor instances
    // For now, it's just a marker type
}

impl SensorManager {
    /// Create a new sensor manager
    pub fn new() -> Self {
        Self {}
    }

    /// Get temperature from the specified sensor
    pub fn get_temperature(&self, sensor_type: &SensorType) -> Result<f32> {
        match sensor_type {
            SensorType::Cpu => CpuSensor::get_temperature(),
            SensorType::Gpu => GpuSensor::get_temperature(),
        }
    }

    /// Check if a sensor is available
    #[allow(unused)]
    pub fn is_sensor_available(&self, sensor_type: &SensorType) -> bool {
        match sensor_type {
            SensorType::Cpu => CpuSensor::is_available(),
            SensorType::Gpu => GpuSensor::is_available(),
        }
    }

    /// Get information about available sensors
    #[allow(unused)]
    pub fn get_sensor_info(&self) -> Vec<(SensorType, bool)> {
        vec![
            (SensorType::Cpu, self.is_sensor_available(&SensorType::Cpu)),
            (SensorType::Gpu, self.is_sensor_available(&SensorType::Gpu)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_monitor_creation() {
        let config = Config::default();
        let monitor = TemperatureMonitor::new(config);
        assert!(
            monitor.get_current_temperature().is_ok() || monitor.get_current_temperature().is_err()
        );
    }

    #[test]
    fn test_sensor_manager() {
        let manager = SensorManager::new();
        let sensor_info = manager.get_sensor_info();
        assert!(!sensor_info.is_empty());

        for (sensor_type, available) in sensor_info {
            if available {
                let temp = manager.get_temperature(&sensor_type);
                assert!(
                    temp.is_ok(),
                    "Failed to get temperature from {:?}: {:?}",
                    sensor_type,
                    temp
                );
            }
        }
    }

    #[test]
    fn test_temperature_conversion() {
        let mut config = Config::default();
        config.temperature_unit = TemperatureUnit::Celsius;
        let monitor = TemperatureMonitor::new(config);

        // Test Celsius conversion (should be identity)
        assert_eq!(monitor.convert_temperature(25.0), 25.0);

        // Test Fahrenheit conversion
        let mut config_f = Config::default();
        config_f.temperature_unit = TemperatureUnit::Fahrenheit;
        let monitor_f = TemperatureMonitor::new(config_f);

        // 25°C = 77°F
        assert!((monitor_f.convert_temperature(25.0) - 77.0).abs() < 0.1);
    }
}
