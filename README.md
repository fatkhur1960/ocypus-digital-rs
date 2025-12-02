# Ocypus L24 Digital

A Rust application that monitors system temperature and displays it on an Ocypus Iota L24 USB device with advanced features including configuration, logging, and automatic reconnection.

## Description

This program connects to an Ocypus Iota L24 digital display device via USB HID interface and continuously monitors system temperature, displaying the current temperature on the device. The application features:

- Automatic device detection and connection (VID: 0x1a2c, PID: 0x434d)
- Configurable temperature units (Celsius/Fahrenheit)
- Configurable monitoring interval
- Temperature threshold alerts
- Automatic device reconnection on disconnection
- Comprehensive logging with multiple verbosity levels
- Configuration file support (TOML format)
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
- `serde` & `serde_json` - Configuration file parsing
- `log` & `env_logger` - Structured logging
- `clap` - Command line argument parsing
- `toml` - TOML configuration file support

## Usage

### Command Line Options

```bash
# Run with default settings
ocypus-l24-digital

# Specify configuration file
ocypus-l24-digital --config /path/to/config.toml

# Set log level
ocypus-l24-digital --log-level debug

# Show help
ocypus-l24-digital --help
```

### Configuration

The application uses a TOML configuration file. Default location: `/etc/ocypus-digital/config.toml`

```toml
# Temperature unit: 'c' for Celsius, 'f' for Fahrenheit
unit = 'c'

# Temperature update interval in seconds
interval = 1

# High temperature threshold for alerts (°C)
high_threshold = 80.0

# Low temperature threshold for alerts (°C)
low_threshold = 20.0

# Enable temperature threshold alerts
alerts = false

# Temperature sensor to use ('cpu', 'system')
sensor = 'cpu'
```

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
- Configuration parsing
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