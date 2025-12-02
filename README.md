# Ocypus L24 Digital

A Rust application that monitors CPU temperature and displays it on an Ocypus Iota L24 USB device.

## Description

This program connects to an Ocypus Iota L24 digital display device via USB HID interface and continuously monitors the system's CPU temperature, displaying the current temperature on the device. The application:

- Automatically detects and connects to the Ocypus Iota L24 device (VID: 0x1a2c, PID: 0x434d)
- Monitors CPU temperature every second using system statistics
- Sends temperature data to the device in the proper HID report format
- Handles temperature conversion and display formatting (supports Celsius and potentially Fahrenheit)

## Installation

### Prerequisites

- Rust 2024 edition or later
- Linux system with USB HID support
- Ocypus Iota L24 device

### Build from source

```bash
git clone https://github.com/fatkhur1960/ocypus-digital-rs.git
cd ocypus-digital-rs
cargo build --release
```

### Dependencies

The project uses the following Rust crates:
- `hidapi` v2.6.3 - USB HID device communication
- `systemstat` v0.2.5 - System statistics and CPU temperature monitoring

## Usage

```bash
# Run the application
cargo run

# Or run the release binary
./target/release/ocypus-l24-digital
```

The application will:
1. Scan for available HID devices
2. Connect to the Ocypus Iota L24 device
3. Start monitoring CPU temperature
4. Display temperature readings on both console and the device

## Device Communication

The program communicates with the Ocypus device using HID reports:
- Report ID: 0x07
- Report length: 64 bytes
- Temperature format: Hundreds, tens, and ones digits in bytes 3-5

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
- [ ] Add error handling for device disconnection
- [ ] Implement Fahrenheit temperature display option
- [ ] Add configuration file support for device settings
- [ ] Create systemd service file for auto-start
- [ ] Add logging with different verbosity levels
- [ ] Implement temperature threshold alerts
- [ ] Add support for other temperature sensors
- [ ] Create installation script
- [ ] Add unit tests for temperature conversion functions