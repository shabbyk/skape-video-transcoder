use std::process::Command;

pub fn detect_gpu_type() -> &'static str {
    let output = Command::new("ffmpeg").arg("-encoders").output();

    if std::env::var("FORCE_CPU").is_ok() {
        println!("🔧 FORCE_CPU set — skipping GPU detection");
        return "cpu";
    }

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("h264_nvenc") {
            println!("⚡ NVIDIA GPU (NVENC) detected");
            return "nvenc";
        } else if stdout.contains("h264_vaapi") {
            println!("🔌 VAAPI GPU detected");
            return "vaapi";
        }
    }

    println!("🧱 No GPU detected, defaulting to VAAPI");
    "cpu"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_gpu_type_returns_valid_option() {
        let result = detect_gpu_type();
        assert!(
            ["nvenc", "vaapi", "cpu"].contains(&result),
            "Unexpected GPU type: {}",
            result
        );
    }

    #[test]
    fn test_force_cpu_env_overrides_detection() {
        // Set the FORCE_CPU env var
        std::env::set_var("FORCE_CPU", "1");

        let result = detect_gpu_type();
        assert_eq!(
            result, "cpu",
            "FORCE_CPU was set, but result was {}",
            result
        );

        // Clean up env var for other tests
        std::env::remove_var("FORCE_CPU");
    }
}
