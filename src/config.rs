use crate::error::{OcypusError, Result};
use clap::Parser;
use std::time::Duration;

/// Device constants
pub const VID: u16 = 0x1a2c;
pub const PID: u16 = 0x434d;
pub const REPORT_ID: u8 = 0x07;
pub const REPORT_LENGTH: usize = 64;

/// Temperature unit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    pub fn from_char(c: char) -> Result<Self> {
        match c.to_lowercase().next() {
            Some('c') => Ok(TemperatureUnit::Celsius),
            Some('f') => Ok(TemperatureUnit::Fahrenheit),
            _ => Err(OcypusError::Config(format!(
                "Invalid temperature unit: '{}'. Use 'c' or 'f'",
                c
            ))),
        }
    }

    pub fn as_char(self) -> char {
        match self {
            TemperatureUnit::Celsius => 'C',
            TemperatureUnit::Fahrenheit => 'F',
        }
    }
}

/// Sensor type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensorType {
    Cpu,
    Gpu,
}

impl SensorType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "cpu" => Ok(SensorType::Cpu),
            "gpu" => Ok(SensorType::Gpu),
            _ => Err(OcypusError::InvalidSensorType(s.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SensorType::Cpu => "cpu",
            SensorType::Gpu => "gpu",
        }
    }
}

/// Command line arguments
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Temperature monitoring utility for Ocypus Iota L24 display",
    long_about = "A modern Rust application that monitors system temperature and displays it on an Ocypus Iota L24 digital display."
)]
pub struct Args {
    /// Temperature unit: 'c' for Celsius, 'f' for Fahrenheit
    #[arg(short, long, default_value = "c")]
    pub unit: char,

    /// Temperature update interval in seconds
    #[arg(short, long, default_value = "1")]
    pub interval: u64,

    /// High temperature threshold for alerts (°C)
    #[arg(long, default_value = "80.0")]
    pub high_threshold: f32,

    /// Low temperature threshold for alerts (°C)
    #[arg(long, default_value = "20.0")]
    pub low_threshold: f32,

    /// Enable temperature threshold alerts
    #[arg(long)]
    pub alerts: bool,

    /// Temperature sensor to use ('cpu', 'gpu')
    #[arg(short, long, default_value = "cpu")]
    pub sensor: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub temperature_unit: TemperatureUnit,
    pub update_interval: Duration,
    pub high_threshold: f32,
    pub low_threshold: f32,
    pub alerts_enabled: bool,
    pub sensor_type: SensorType,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            temperature_unit: TemperatureUnit::Celsius,
            update_interval: Duration::from_secs(1),
            high_threshold: 80.0,
            low_threshold: 20.0,
            alerts_enabled: false,
            sensor_type: SensorType::Cpu,
        }
    }
}

impl Config {
    /// Create configuration from command line arguments
    pub fn from_args(args: &Args) -> Result<Self> {
        Ok(Config {
            temperature_unit: TemperatureUnit::from_char(args.unit)?,
            update_interval: Duration::from_secs(args.interval),
            high_threshold: args.high_threshold,
            low_threshold: args.low_threshold,
            alerts_enabled: args.alerts,
            sensor_type: SensorType::from_str(&args.sensor)?,
        })
    }

    /// Validate configuration and return self for chaining
    pub fn validate(self) -> Result<Self> {
        if self.high_threshold <= self.low_threshold {
            return Err(OcypusError::Config(
                "High threshold must be greater than low threshold".to_string(),
            ));
        }

        if self.update_interval.as_secs() == 0 {
            return Err(OcypusError::Config(
                "Update interval must be greater than 0 seconds".to_string(),
            ));
        }

        Ok(self)
    }
}