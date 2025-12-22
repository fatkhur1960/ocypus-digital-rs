# Project Structure

This project follows modern Rust best practices with a modular architecture:

## Directory Structure

```
src/
├── main.rs              # Application entry point
├── config.rs           # Configuration management and CLI arguments
├── error.rs            # Error handling with thiserror
├── device.rs           # Device communication (HID API)
├── monitor.rs          # Temperature monitoring service
└── sensor/             # Sensor modules
    ├── mod.rs
    ├── cpu_sensor.rs   # CPU temperature sensor
    └── gpu_sensor.rs   # GPU temperature sensor
```

## Architecture Overview

### Configuration (`config.rs`)
- CLI argument parsing with clap
- Configuration validation
- Temperature unit and sensor type enums
- Device constants

### Error Handling (`error.rs`)
- Centralized error types using thiserror
- Result type alias for convenience
- Specific error categories (Device, Sensor, Config, etc.)

### Device Management (`device.rs`)
- HID device communication
- Connection management and reconnection
- Temperature report building and sending
- Error handling for device operations

### Temperature Monitoring (`monitor.rs`)
- Temperature monitoring service
- Sensor management
- Threshold checking and alerts
- Temperature conversion between units

### Sensors (`sensor/`)
- **CPU Sensor**: Uses lm-sensors for temperature readings
- **GPU Sensor**: Supports NVIDIA (nvidia-smi), AMD (amd-smi/rocm-smi), and lm-sensors fallback
- Both sensors provide availability checking and robust error handling

### Main Application (`main.rs`)
- Application lifecycle management
- Logging setup
- Configuration validation
- Main event loop with error recovery

## Key Features

- **Modular Design**: Clear separation of concerns
- **Robust Error Handling**: Comprehensive error types and recovery
- **Type Safety**: Strong typing with enums for configuration
- **Testable Architecture**: Each module is unit tested
- **Modern Rust**: Uses 2021 edition with current best practices
- **Async-Ready**: Optional async support via feature flags

## Dependencies

- `clap`: CLI argument parsing with derive macros
- `hidapi`: Hardware device communication
- `thiserror`: Error handling
- `regex`: Text parsing for sensor outputs
- `log` + `env_logger`: Structured logging
- Optional: `tokio` for async operations

## Testing

The project includes comprehensive unit tests for all major components:

```bash
cargo test                    # Run all tests
cargo test --module sensor     # Test sensor modules only
cargo test --module device     # Test device communication
```

## CLI Usage

```bash
# Basic usage
ocypus-digital --sensor cpu --unit c --interval 2

# With alerts
ocypus-digital --sensor gpu --alerts --high-threshold 85

# Debug logging
ocypus-digital --log-level debug --sensor cpu
```