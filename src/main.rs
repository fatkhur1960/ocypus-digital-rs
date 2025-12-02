extern crate hidapi;
extern crate systemstat;

use std::{sync::mpsc, thread};

use hidapi::HidApi;
use systemstat::{Platform, System};

const VID: u16 = 0x1a2c;
const PID: u16 = 0x434d;

const REPORT_ID: u8 = 0x07;
const REPORT_LENGTH: usize = 64;

fn main() {
    match HidApi::new() {
        Ok(api) => {
            println!("Available devices:");

            // Enumerate devices and open the first matching one
            let mut device_opt: Option<hidapi::HidDevice> = None;

            for device_info in api.device_list() {
                if device_info.vendor_id() == VID && device_info.product_id() == PID {
                    let path = device_info.path().to_string_lossy().into_owned();
                    print!(" - {}", path);

                    match api.open_path(device_info.path()) {
                        Ok(dev) => {
                            println!(" (opened)");
                            device_opt = Some(dev);
                            break;
                        }
                        Err(_) => {
                            println!(" (failed to open)");
                        }
                    }
                }
            }

            let dev = match device_opt {
                Some(hid_device) => hid_device,
                None => {
                    eprintln!("No Ocypus Iota L24 device found.");
                    std::process::exit(1);
                }
            };

            println!("Connected to Ocypus Iota L24");
            println!("Monitoring CPU temp...");

            let (tx, rx) = mpsc::channel::<f32>();

            thread::spawn(move || {
                loop {
                    let sys = System::new();
                    if let Ok(temp) = sys.cpu_temp() {
                        if let Err(e) = tx.send(temp) {
                            eprintln!("Failed to send temp to device: {e}");
                        }
                    }

                    thread::sleep(std::time::Duration::from_millis(1000));
                }
            });

            for temp in rx {
                if let Err(e) = send_temp(&dev, temp) {
                    eprintln!("Failed to send temp to device: {e}");
                } else {
                    println!("Temp: {temp:.0}°C");
                }
            }
        }
        Err(e) => println!("No Ocypus device found: {}", e),
    }
}

// Build HID report like the Python build_report()
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

fn send_temp(dev: &hidapi::HidDevice, temp_celsius: f32) -> hidapi::HidResult<usize> {
    let data = build_report(temp_celsius, 'c');
    dev.write(&data)
}
