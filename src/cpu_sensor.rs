use std::{
    io::{Error, ErrorKind},
    process::Command,
};

use regex::Regex;

// get the CPU temperature with the best available method
pub fn get_cpu_temperature() -> Result<f32, Error> {
    let out = Command::new("sensors")
        .output()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    let text = String::from_utf8_lossy(&out.stdout);

    // Look for the temperature in the output
    let patterns = [
        r"Package id 0:\s*\+([0-9]+(?:\.[0-9]+)?)°C", // Intel
        r"Tdie:\s*\+([0-9]+(?:\.[0-9]+)?)°C",         // AMD real die temp
        r"Tctl:\s*\+([0-9]+(?:\.[0-9]+)?)°C",         // AMD control temp
        r"temp1:\s*\+([0-9]+(?:\.[0-9]+)?)°C",        // fallback
    ];

    for pat in patterns {
        let re = Regex::new(pat).unwrap();
        if let Some(cap) = re.captures(&text) {
            return cap[1]
                .parse::<f32>()
                .map_err(|e| Error::new(ErrorKind::Other, e));
        }
    }

    Err(Error::new(ErrorKind::NotFound, "CPU temperature not found"))
}
