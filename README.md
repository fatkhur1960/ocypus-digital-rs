# Ocypus L24 Digital

A Rust application that monitors system temperature and displays it on an Ocypus Iota L24 USB device with advanced features including configuration, logging, and automatic reconnection.

## Description

This program connects to an Ocypus Iota L24 digital display device via USB HID interface and continuously monitors system temperature, displaying the current temperature on the device. The application features:

- Automatic device detection and connection (VID: 0x1a2c, PID: 0x434d)
- Configurable temperature units (Celsius/Fahrenheit) via CLI
- Configurable monitoring interval via CLI
- Temperature threshold alerts via CLI
- Automatic device reconnection on disconnection
- Comprehensive logging with multiple verbosity levels
- Command-line interface for all configuration options
- Multiple temperature sensor support
- Unit tests for core functionality
- Systemd service integration

## Installation

### Prerequisites

- Rust 2024 edition or later
- Linux system with USB HID support
- Ocypus Iota L24 device

### Automated Installation

```bash
git clone https://github.com/fatkhur1960/ocypus-digital-rs.git
cd ocypus-digital-rs
./install.sh
```

The installation script will:
- Build the application
- Install the binary to `/usr/local/bin`
- Create default configuration at `/etc/ocypus-digital/config.toml`
- Install and enable systemd service
- Set up USB device permissions via udev rules

### Manual Installation

```bash
git clone https://github.com/fatkhur1960/ocypus-digital-rs.git
cd ocypus-digital-rs
cargo build --release
sudo cp target/release/ocypus-l24-digital /usr/local/bin/
sudo cp ocypus-digital.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable ocypus-digital.service
```

### Dependencies

The project uses the following Rust crates:
- `hidapi` v2.6.3 - USB HID device communication
- `systemstat` v0.2.5 - System statistics and temperature monitoring
- `log` & `env_logger` - Structured logging
- `clap` - Command line argument parsing

## Usage

### Command Line Options

```bash
# Run with default settings
ocypus-l24-digital

# Set temperature unit
ocypus-l24-digital --unit f

# Set update interval
ocypus-l24-digital --interval 2

# Enable temperature alerts
ocypus-l24-digital --alerts --high-threshold 75.0 --low-threshold 25.0

# Set sensor type
ocypus-l24-digital --sensor cpu

# Set log level
ocypus-l24-digital --log-level debug

# Show help
ocypus-l24-digital --help

# Example with all options
ocypus-l24-digital --unit f --interval 2 --alerts --high-threshold 85.0 --low-threshold 15.0 --sensor cpu --log-level info
```

### Configuration Options

All configuration is done via command-line arguments:

- `--unit, -u`: Temperature unit ('c' for Celsius, 'f' for Fahrenheit) [default: c]
- `--interval, -i`: Temperature update interval in seconds [default: 1]
- `--high-threshold`: High temperature threshold for alerts (°C) [default: 80.0]
- `--low-threshold`: Low temperature threshold for alerts (°C) [default: 20.0]
- `--alerts`: Enable temperature threshold alerts
- `--sensor, -s`: Temperature sensor to use ('cpu', 'system') [default: cpu]
- `--log-level, -l`: Log level (trace, debug, info, warn, error) [default: info]

### Systemd Service

```bash
# Start the service
sudo systemctl start ocypus-digital.service

# Enable auto-start on boot
sudo systemctl enable ocypus-digital.service

# Check status
sudo systemctl status ocypus-digital.service

# View logs
sudo journalctl -u ocypus-digital.service -f
```

## Features

### Temperature Units
- **Celsius**: Default unit, displays temperature in °C
- **Fahrenheit**: Automatic conversion from Celsius to °F

### Temperature Sensors
- **CPU**: Monitors CPU temperature (default)
- **System**: Monitors system temperature sensors

### Alerts
- Configurable high and low temperature thresholds
- Console and log warnings when thresholds are exceeded

### Logging
- Multiple log levels: trace, debug, info, warn, error
- Structured logging with timestamps
- Systemd journal integration

### Error Handling
- Automatic device reconnection on disconnection
- Graceful handling of sensor read failures
- Comprehensive error reporting

## Device Communication

The program communicates with the Ocypus device using HID reports:
- Report ID: 0x07
- Report length: 64 bytes
- Temperature format: Hundreds, tens, and ones digits in bytes 3-5

## Testing

Run the unit tests:
```bash
cargo test
```

Tests include:
- Temperature report building
- Unit conversion
- Configuration defaults
- Temperature clamping

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## What to do next

- [x] Create LICENSE file
- [x] Create systemd service file for auto-start
- [x] Add error handling for device disconnection
- [x] Implement Fahrenheit temperature display option
- [x] Add configuration file support for device settings
- [x] Add logging with different verbosity levels
- [x] Implement temperature threshold alerts
- [x] Add support for other temperature sensors
- [x] Create installation script
- [x] Add unit tests for temperature conversion functions

All planned features have been implemented! 🎉