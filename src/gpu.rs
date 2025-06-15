use std::process::Command;

pub fn detect_gpu_type() -> &'static str {
    let output = Command::new("ffmpeg").arg("-encoders").output();

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
    "vaapi"
}
