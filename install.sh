#!/bin/bash

# Ocypus L24 Digital Temperature Monitor Installation Script
# This script installs the application and sets up the systemd service

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root for security reasons"
   print_error "Run as regular user and use sudo when prompted"
   exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo is not installed"
    print_status "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    print_status "Rust/Cargo is already installed"
fi

# Build the application
print_status "Building the application..."
cargo build --release

# Install the binary
print_status "Installing binary to /usr/local/bin..."
sudo cp target/release/ocypus-l24-digital /usr/local/bin/
sudo chmod 755 /usr/local/bin/ocypus-l24-digital

# Create config directory
print_status "Creating config directory..."
sudo mkdir -p /etc/ocypus-digital

# Copy default config if it doesn't exist
if [[ ! -f /etc/ocypus-digital/config.toml ]]; then
    print_status "Installing default configuration..."
    sudo tee /etc/ocypus-digital/config.toml > /dev/null <<EOF
# Ocypus L24 Digital Configuration File

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

# Temperature sensor to use ('cpu', 'gpu', 'system')
sensor = 'cpu'
EOF
else
    print_warning "Configuration file already exists at /etc/ocypus-digital/config.toml"
fi

# Install systemd service
print_status "Installing systemd service..."
sudo cp ocypus-digital.service /etc/systemd/system/

# Update service file to use correct paths
sudo sed -i 's|ExecStart=/usr/local/bin/ocypus-l24-digital|ExecStart=/usr/local/bin/ocypus-l24-digital --config /etc/ocypus-digital/config.toml|' /etc/systemd/system/ocypus-digital.service

# Reload systemd and enable service
print_status "Setting up systemd service..."
sudo systemctl daemon-reload
sudo systemctl enable ocypus-digital.service

# Create udev rule for USB device access
print_status "Creating udev rule for USB device access..."
sudo tee /etc/udev/rules.d/99-ocypus-l24.rules > /dev/null <<EOF
# Ocypus Iota L24 USB device access rule
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1a2c", ATTRS{idProduct}=="434d", MODE="0666", GROUP="plugdev"
EOF

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

print_status "Installation completed successfully!"
print_status ""
print_status "Next steps:"
print_status "1. Edit configuration: sudo nano /etc/ocypus-digital/config.toml"
print_status "2. Start the service: sudo systemctl start ocypus-digital.service"
print_status "3. Check status: sudo systemctl status ocypus-digital.service"
print_status "4. View logs: sudo journalctl -u ocypus-digital.service -f"
print_status ""
print_status "The service will automatically start on boot."

# Ask if user wants to start the service now
read -p "Do you want to start the service now? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Starting ocypus-digital service..."
    sudo systemctl start ocypus-digital.service
    sleep 2
    
    if sudo systemctl is-active --quiet ocypus-digital.service; then
        print_status "Service started successfully!"
        print_status "Current status:"
        sudo systemctl status ocypus-digital.service --no-pager
    else
        print_error "Failed to start service. Check logs with: sudo journalctl -u ocypus-digital.service"
    fi
fi