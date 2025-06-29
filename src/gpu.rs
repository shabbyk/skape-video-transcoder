use std::process::{Command};

use std::fs;

/// Detects GPU type based on Linux device files.
///
/// Returns:
/// - "nvenc"  if NVIDIA GPU is detected
/// - "vaapi"  if Intel or AMD GPU is detected
/// - "cpu"    if no GPU is detected
pub fn detect_gpu_from_devices() -> &'static str {
    // Allow override
    if std::env::var("FORCE_CPU").is_ok() {
        println!("🔧 FORCE_CPU set — skipping GPU detection");
        return "cpu";
    }

    // Check for Intel/AMD GPUs via /dev/dri
    if let Ok(entries) = fs::read_dir("/dev/dri") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name()
                   .and_then(|n| n.to_str())
                   .map_or(false, |name| name.starts_with("card") || name.starts_with("renderD")) {
                println!("🔌 VAAPI-compatible GPU detected: {:?}", path);
                return "vaapi";
            }
        }
    } else {
        println!("❌ Cannot access /dev/dri");
    }

    // Check for NVIDIA GPU devices in /dev
    if let Ok(entries) = fs::read_dir("/dev") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("nvidia") {
                    println!("⚡ NVIDIA GPU device detected: {:?}", entry.path());
                    return "nvenc";
                }
            }
        }
    } else {
        println!("❌ Cannot access /dev");
    }

    println!("🧱 No GPU detected via device files, defaulting to CPU");
    "cpu"
}

//TODO: Work in progress
pub fn detect_gpu_type() -> &'static str {
    if std::env::var("FORCE_CPU").is_ok() {
        println!("🔧 FORCE_CPU set — skipping GPU detection");
        return "cpu";
    }

    // Step 1: Try detecting NVIDIA GPU via CUDA init
    let nvidia = Command::new("ffmpeg")
        .args(&[
            "-init_hw_device", "cuda=cu:0",
            "-f", "lavfi",
            "-i", "nullsrc",
            "-frames:v", "1",
            "-f", "null", "-"
        ])
        //.stdout(Stdio::null())
        //.stderr(Stdio::piped())
        .output();

    if let Ok(output) = nvidia {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.to_lowercase().contains("cuda device") {
            println!("⚡ NVIDIA GPU (CUDA) detected");
            return "nvenc";
        }
    }

    // Step 2: Try detecting Intel/AMD via VAAPI
    let vaapi = Command::new("ffmpeg")
                .args(&[
                    "-hwaccel", "vaapi",
                    "-f", "lavfi",
                    "-i", "nullsrc",
                    "-frames:v", "1",
                    "-f", "null", "-"
                ])
        //.stdout(Stdio::null())
        //.stderr(Stdio::piped())
        .output();

    if let Ok(output) = vaapi {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("/dev/dri/renderD") || stderr.to_lowercase().contains("vaapi") {
            println!("🔌 VAAPI GPU detected");
            return "vaapi";
        }
    }

    println!("🧱 No GPU detected, defaulting to CPU");
    "cpu"
}

// TODO: Add unit test (mocking is an over-head), integration test seems simpler