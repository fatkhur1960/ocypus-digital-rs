use thiserror::Error;

/// Main error type for the ocypus-digital application
#[derive(Error, Debug)]
pub enum OcypusError {
    /// Device-related errors
    #[error("Device error: {0}")]
    Device(String),

    /// Sensor-related errors
    #[error("Sensor error: {0}")]
    Sensor(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// HID API errors
    #[error("HID error: {0}")]
    HidApi(String),

    /// Temperature parsing errors
    #[error("Failed to parse temperature: {0}")]
    TemperatureParse(String),

    /// Invalid sensor type
    #[error("Invalid sensor type: '{0}'. Supported types: cpu, gpu")]
    InvalidSensorType(String),
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, OcypusError>;