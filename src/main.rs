extern crate hidapi;
extern crate systemstat;

use std::{sync::mpsc, thread, time::Duration, fs, process};
use std::path::Path;
use hidapi::HidApi;
use systemstat::{Platform, System};
use log::{info, warn, error, debug};
use clap::Parser;
use serde::{Deserialize, Serialize};

const VID: u16 = 0x1a2c;
const PID: u16 = 0x434d;
const REPORT_ID: u8 = 0x07;
const REPORT_LENGTH: usize = 64;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Config {
    /// Temperature unit: 'c' for Celsius, 'f' for Fahrenheit
    #[serde(default = "default_unit")]
    unit: char,
    
    /// Temperature update interval in seconds
    #[serde(default = "default_interval")]
    interval: u64,
    
    /// High temperature threshold for alerts
    #[serde(default = "default_high_threshold")]
    high_threshold: f32,
    
    /// Low temperature threshold for alerts
    #[serde(default = "default_low_threshold")]
    low_threshold: f32,
    
    /// Enable temperature threshold alerts
    #[serde(default = "default_alerts")]
    alerts: bool,
    
    /// Temperature sensor to use ('cpu', 'gpu', 'system')
    #[serde(default = "default_sensor")]
    sensor: String,
}

fn default_unit() -> char { 'c' }
fn default_interval() -> u64 { 1 }
fn default_high_threshold() -> f32 { 80.0 }
fn default_low_threshold() -> f32 { 20.0 }
fn default_alerts() -> bool { false }
fn default_sensor() -> String { "cpu".to_string() }

impl Default for Config {
    fn default() -> Self {
        Config {
            unit: default_unit(),
            interval: default_interval(),
            high_threshold: default_high_threshold(),
            low_threshold: default_low_threshold(),
            alerts: default_alerts(),
            sensor: default_sensor(),
        }
    }
}

fn load_config(config_path: &str) -> Config {
    if Path::new(config_path).exists() {
        match fs::read_to_string(config_path) {
            Ok(content) => {
                match toml::from_str(&content) {
                    Ok(config) => {
                        info!("Loaded configuration from {}", config_path);
                        return config;
                    }
                    Err(e) => {
                        warn!("Failed to parse config file {}: {}", config_path, e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read config file {}: {}", config_path, e);
            }
        }
    } else {
        info!("Config file {} not found, using defaults", config_path);
        // Create default config file
        let default_config = Config::default();
        if let Ok(content) = toml::to_string_pretty(&default_config) {
            if let Err(e) = fs::write(config_path, content) {
                warn!("Failed to create default config file: {}", e);
            } else {
                info!("Created default config file: {}", config_path);
            }
        }
    }
    
    Config::default()
}

fn get_temperature(sys: &System, sensor: &str) -> Option<f32> {
    match sensor {
        "cpu" => {
            match sys.cpu_temp() {
                Ok(temp) => {
                    debug!("CPU temperature: {:.1}°C", temp);
                    Some(temp)
                }
                Err(e) => {
                    warn!("Failed to get CPU temperature: {}", e);
                    None
                }
            }
        }
        "system" => {
            // systemstat doesn't have a generic temp() method, fallback to CPU temp
            match sys.cpu_temp() {
                Ok(temp) => {
                    debug!("System temperature (CPU fallback): {:.1}°C", temp);
                    Some(temp)
                }
                Err(e) => {
                    warn!("Failed to get system temperature: {}", e);
                    None
                }
            }
        }
        _ => {
            warn!("Unknown sensor type: {}", sensor);
            None
        }
    }
}

fn check_thresholds(temp: f32, config: &Config) {
    if !config.alerts {
        return;
    }
    
    if temp > config.high_threshold {
        warn!("High temperature alert: {:.1}°C (threshold: {:.1}°C)", 
              temp, config.high_threshold);
    } else if temp < config.low_threshold {
        warn!("Low temperature alert: {:.1}°C (threshold: {:.1}°C)", 
              temp, config.low_threshold);
    }
}

fn build_report(temp_celsius: f32, unit: char) -> [u8; REPORT_LENGTH] {
    // Clamp temp in °C between 0 and 99
    let mut temp_c = temp_celsius as i32;
    if temp_c < 0 {
        temp_c = 0;
    } else if temp_c > 99 {
        temp_c = 99;
    }

    // Convert to display temp, optionally in °F
    let mut display_temp = if unit == 'f' || unit == 'F' {
        (temp_c as f32 * 9.0 / 5.0 + 32.0).round() as i32
    } else {
        temp_c
    };

    // Clamp display_temp (like Python: 0..212)
    if display_temp < 0 {
        display_temp = 0;
    } else if display_temp > 212 {
        display_temp = 212;
    }

    let hundreds = display_temp / 100;
    let tens = (display_temp % 100) / 10;
    let ones = display_temp % 10;

    let mut report = [0u8; REPORT_LENGTH];
    report[0] = REPORT_ID;
    report[1] = 0xff;
    report[2] = 0xff;
    report[3] = hundreds as u8;
    report[4] = tens as u8;
    report[5] = ones as u8;

    report
}

fn send_temp(dev: &hidapi::HidDevice, temp_celsius: f32, unit: char) -> Result<usize, String> {
    let data = build_report(temp_celsius, unit);
    match dev.write(&data) {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(format!("Failed to write to device: {}", e)),
    }
}

fn connect_device(api: &HidApi) -> Result<hidapi::HidDevice, String> {
    info!("Scanning for Ocypus Iota L24 device...");
    
    for device_info in api.device_list() {
        if device_info.vendor_id() == VID && device_info.product_id() == PID {
            let path = device_info.path().to_string_lossy().into_owned();
            debug!("Found device at: {}", path);
            
            match api.open_path(device_info.path()) {
                Ok(dev) => {
                    info!("Connected to Ocypus Iota L24 at {}", path);
                    return Ok(dev);
                }
                Err(e) => {
                    warn!("Failed to open device at {}: {}", path, e);
                }
            }
        }
    }
    
    Err("No Ocypus Iota L24 device found".to_string())
}

fn main() {
    let args = Args::parse();
    
    // Initialize logging
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
    
    // Load configuration
    let config = load_config(&args.config);
    info!("Using temperature unit: {}°", config.unit.to_uppercase());
    info!("Update interval: {} seconds", config.interval);
    if config.alerts {
        info!("Temperature alerts enabled (high: {:.1}°C, low: {:.1}°C)", 
              config.high_threshold, config.low_threshold);
    }
    
    match HidApi::new() {
        Ok(api) => {
            let mut device = match connect_device(&api) {
                Ok(dev) => dev,
                Err(e) => {
                    error!("Failed to connect to device: {}", e);
                    process::exit(1);
                }
            };
            
            info!("Starting temperature monitoring...");
            
            let (tx, rx) = mpsc::channel::<f32>();
            
            // Temperature monitoring thread
            let config_clone = config.clone();
            thread::spawn(move || {
                let sys = System::new();
                loop {
                    if let Some(temp) = get_temperature(&sys, &config_clone.sensor) {
                        check_thresholds(temp, &config_clone);
                        if let Err(e) = tx.send(temp) {
                            error!("Failed to send temperature: {}", e);
                            break;
                        }
                    }
                    
                    thread::sleep(Duration::from_secs(config_clone.interval));
                }
            });
            
            // Main loop - handle device communication and reconnection
            for temp in rx {
                match send_temp(&device, temp, config.unit) {
                    Ok(_) => {
                        let display_temp = if config.unit == 'f' {
                            temp * 9.0 / 5.0 + 32.0
                        } else {
                            temp
                        };
                        info!("Temperature: {:.0}°{}", display_temp, config.unit.to_uppercase());
                    }
                    Err(e) => {
                        error!("Device communication error: {}", e);
                        info!("Attempting to reconnect...");
                        
                        // Try to reconnect
                        match connect_device(&api) {
                            Ok(new_device) => {
                                device = new_device;
                                info!("Successfully reconnected to device");
                            }
                            Err(e) => {
                                error!("Reconnection failed: {}", e);
                                thread::sleep(Duration::from_secs(5));
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to initialize HID API: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_report_celsius() {
        let report = build_report(25.5, 'c');
        assert_eq!(report[0], REPORT_ID);
        assert_eq!(report[3], 0); // hundreds
        assert_eq!(report[4], 2);  // tens
        assert_eq!(report[5], 5);  // ones
    }
    
    #[test]
    fn test_build_report_fahrenheit() {
        let report = build_report(25.0, 'f'); // 25°C = 77°F
        assert_eq!(report[0], REPORT_ID);
        assert_eq!(report[3], 0); // hundreds
        assert_eq!(report[4], 7);  // tens
        assert_eq!(report[5], 7);  // ones
    }
    
    #[test]
    fn test_build_report_clamping() {
        // Test negative temperature
        let report = build_report(-10.0, 'c');
        assert_eq!(report[3], 0);
        assert_eq!(report[4], 0);
        assert_eq!(report[5], 0);
        
        // Test high temperature
        let report = build_report(150.0, 'c');
        assert_eq!(report[3], 0);
        assert_eq!(report[4], 9);
        assert_eq!(report[5], 9);
    }
    
    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.unit, 'c');
        assert_eq!(config.interval, 1);
        assert_eq!(config.high_threshold, 80.0);
        assert_eq!(config.low_threshold, 20.0);
        assert!(!config.alerts);
        assert_eq!(config.sensor, "cpu");
    }
}