mod config;
mod device;
mod error;
mod monitor;
mod sensor;

use clap::Parser;
use config::{Args, Config};
use device::DeviceManager;
use error::Result;
use log::{error, info};
use monitor::TemperatureMonitor;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // Initialize logging first
    setup_logging();

    // Parse command line arguments
    let args = Args::parse();

    // Create and validate configuration
    let config = match Config::from_args(&args).and_then(|c| c.validate()) {
        Ok(config) => config,
        Err(e) => {
            error!("Configuration error: {}", e);
            process::exit(1);
        }
    };

    // Print configuration
    print_config(&config);

    // Run the application
    if let Err(e) = run_application(&config) {
        error!("Application error: {}", e);
        process::exit(1);
    }
}

/// Setup logging based on configuration
fn setup_logging() {
    let args = Args::parse();

    env_logger::Builder::from_default_env()
        .filter_level(match args.log_level.as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            _ => log::LevelFilter::Info,
        })
        .init();
}

/// Print the current configuration
fn print_config(config: &Config) {
    info!("ocypus-digital v{}", env!("CARGO_PKG_VERSION"));
    info!(
        "Using temperature unit: °{}",
        config.temperature_unit.as_char()
    );
    info!(
        "Update interval: {} seconds",
        config.update_interval.as_secs()
    );
    info!("Using sensor: {}", config.sensor_type.as_str());

    if config.alerts_enabled {
        info!(
            "Temperature alerts enabled (high: {:.1}°C, low: {:.1}°C)",
            config.high_threshold, config.low_threshold
        );
    }
}

/// Main application logic
fn run_application(config: &Config) -> Result<()> {
    // Initialize HID API and device manager
    let mut device_manager = DeviceManager::new()?;

    // Connect to the device
    device_manager.connect()?;

    // Initialize temperature monitor
    let temperature_monitor = TemperatureMonitor::new(config.clone());

    // Start temperature monitoring in a separate thread
    let temp_receiver = temperature_monitor.start_monitoring()?;

    // Main application loop
    main_loop(&mut device_manager, &temperature_monitor, temp_receiver)
}

/// Main application loop
fn main_loop(
    device_manager: &mut DeviceManager,
    temperature_monitor: &TemperatureMonitor,
    temp_receiver: mpsc::Receiver<f32>,
) -> Result<()> {
    info!("Starting temperature monitoring loop");

    for temp_celsius in temp_receiver {
        match device_manager
            .send_temperature(temp_celsius, temperature_monitor.config().temperature_unit)
        {
            Ok(_) => {
                let display_temp = temperature_monitor.convert_temperature(temp_celsius);
                info!(
                    "Temperature: {:.0}°{}",
                    display_temp,
                    temperature_monitor.config().temperature_unit.as_char()
                );
            }
            Err(e) => {
                error!("Device communication error: {}", e);

                // Attempt reconnection
                info!("Attempting to reconnect...");
                match device_manager.reconnect() {
                    Ok(_) => {
                        info!("Successfully reconnected to device");
                        // Try to send the failed temperature again
                        if let Err(retry_err) = device_manager.send_temperature(
                            temp_celsius,
                            temperature_monitor.config().temperature_unit,
                        ) {
                            error!(
                                "Failed to send temperature after reconnection: {}",
                                retry_err
                            );
                        }
                    }
                    Err(reconnect_err) => {
                        error!("Failed to reconnect: {}", reconnect_err);
                        // Wait before retrying
                        thread::sleep(Duration::from_secs(5));
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::TemperatureUnit;

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.clone().validate().is_ok());

        // Test invalid thresholds
        let mut config = Config::default();
        config.high_threshold = 50.0;
        config.low_threshold = 60.0;
        assert!(config.validate().is_err());

        // Test invalid interval
        let mut config = Config::default();
        config.update_interval = Duration::from_secs(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_temperature_conversion() {
        let mut config = Config::default();
        config.temperature_unit = TemperatureUnit::Fahrenheit;
        let monitor = TemperatureMonitor::new(config);

        // Test known conversion: 0°C = 32°F
        let fahrenheit = monitor.convert_temperature(0.0);
        assert!((fahrenheit - 32.0).abs() < 0.1);

        // Test known conversion: 100°C = 212°F
        let fahrenheit = monitor.convert_temperature(100.0);
        assert!((fahrenheit - 212.0).abs() < 0.1);
    }

    #[test]
    fn test_sensor_availability() {
        let _monitor = TemperatureMonitor::new(Config::default());
        let sensor_manager = monitor::SensorManager::new();
        let sensor_info = sensor_manager.get_sensor_info();

        // Should have at least CPU sensor listed
        assert!(!sensor_info.is_empty());
    }
}
