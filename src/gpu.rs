use std::process::{Command, Stdio};

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
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
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
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
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