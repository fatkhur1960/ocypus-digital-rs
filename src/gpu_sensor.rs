use std::{io::Error, process::Command};

// get the GPU temperature with the best available method
pub fn get_gpu_temperature() -> Result<f32, Error> {
    gpu_temp_nvidia()
        .or_else(gpu_temp_amd_smi)
        .or_else(gpu_temp_rocm)
        .or_else(gpu_temp_sensors)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Error::new(
            std::io::ErrorKind::Other,
            "GPU temperature not found or parsing failed",
        ))
}

// run a command
fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

// NVIDIA
fn gpu_temp_nvidia() -> Option<String> {
    let out = run(
        "nvidia-smi",
        &[
            "--query-gpu=temperature.gpu",
            "--format=csv,noheader,nounits",
        ],
    )?;

    out.lines().next().map(|s| s.trim().to_string())
}
// AMD (New ROCm)
fn gpu_temp_amd_smi() -> Option<String> {
    let out = run("amd-smi", &["metric", "--temperature"])?;

    out.lines().find_map(|l| {
        if l.to_lowercase().contains("edge") {
            extract_number(l)
        } else {
            None
        }
    })
}

// AMD (Old ROCm)
fn gpu_temp_rocm() -> Option<String> {
    let out = run("rocm-smi", &["--showtemp"])?;

    out.lines().find_map(|l| extract_number(l))
}

// sensors fallback
fn gpu_temp_sensors() -> Option<String> {
    let out = run("sensors", &[])?;

    for line in out.lines() {
        let l = line.to_lowercase();
        if l.contains("gpu") || l.contains("edge") || l.contains("junction") {
            if let Some(n) = extract_number(line) {
                return Some(n);
            }
        }
    }
    None
}

// extract first float in string
fn extract_number(s: &str) -> Option<String> {
    let mut buf = String::new();
    let mut seen_digit = false;

    for c in s.chars() {
        if c.is_ascii_digit() || (c == '.' && seen_digit) {
            buf.push(c);
            seen_digit = true;
        } else if seen_digit {
            break;
        }
    }

    if buf.is_empty() { None } else { Some(buf) }
}
