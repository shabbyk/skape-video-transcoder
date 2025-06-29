use crate::gpu::{detect_gpu_from_devices};
use crate::ledger::{append_to_ledger, load_ledger};
use chrono::Local;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;

const TEMP_DIR: &str = "/tmp/video_convert_work";

pub fn process_directory(watch_dir: &str) {
    let (mkv_files, srt_files) = collect_files(watch_dir);
    let ledger = load_ledger();
    let gpu_type = detect_gpu_from_devices();

    fs::create_dir_all(TEMP_DIR).ok();

    mkv_files.par_iter().for_each(|(base, input_file)| {
        if ledger.contains(base) {
            println!("✅ Skipped (already converted): {}", base);
            return;
        }

        let temp_input = Path::new(TEMP_DIR).join(input_file.file_name().unwrap());
        if let Err(e) = fs::copy(&input_file, &temp_input) {
            println!("❌ Failed to copy to temp: {}", e);
            return;
        }

        let output_file = input_file.with_extension("mp4");
        if output_file.exists() {
            println!("🟡 Already exists: {:?}", output_file);
            append_to_ledger(base);
            return;
        }

        let srt_file = srt_files.get(base);
        let log_file = output_file.with_extension("log");

        println!(
            "\n[{}] 🎬 Converting: {:?}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            input_file
        );
        let start_time = std::time::Instant::now();

        let mut command = Command::new("ffmpeg");
        command.arg("-y");

        // Step 1: Add video input
        match gpu_type {
            "nvenc" => {
                command
                    .arg("-hwaccel")
                    .arg("cuda")
                    .arg("-i")
                    .arg(&temp_input);
            }
            "vaapi" => {
                command
                    .arg("-hwaccel")
                    .arg("vaapi")
                    .arg("-vaapi_device")
                    .arg("/dev/dri/renderD128")
                    .arg("-i")
                    .arg(&temp_input);
            }
            _ => {
                println!("⚠️ GPU not available or unsupported, falling back to CPU encoding.");
                command.arg("-i").arg(&temp_input);
            }
        }

        // Step 2: Add subtitle input if available
        if let Some(srt_path) = srt_file {
            let temp_srt = Path::new(TEMP_DIR).join(srt_path.file_name().unwrap());
            match fs::copy(srt_path, &temp_srt) {
                Ok(_) => println!("💬 Subtitle copied to temp: {:?}", temp_srt),
                Err(e) => {
                    println!("❌ Failed to copy subtitle to temp: {}", e);
                    return;
                }
            }

            command.arg("-f").arg("srt").arg("-i").arg(&temp_srt);
        }

        // Step 3: Mapping and codec configuration
        if srt_file.is_some() {
            command
                .arg("-map")
                .arg("0:v:0")
                .arg("-map")
                .arg("0:a?")
                .arg("-map")
                .arg("1:s:0");
        } else {
            println!("🕳️ No subtitle found for: {:?}", input_file);
            command.arg("-map").arg("0:v:0").arg("-map").arg("0:a?");
        }

        // Step 4: Video codec
        match gpu_type {
            "nvenc" => {
                command.arg("-c:v").arg("h264_nvenc");
            }
            "vaapi" => {
                command
                    .arg("-vf")
                    .arg("format=nv12,hwupload")
                    .arg("-c:v")
                    .arg("h264_vaapi");
            }
            _ => {
                command.arg("-c:v").arg("libx264");
            }
        }

        // Subtitle codec if present
        if srt_file.is_some() {
            command
                .arg("-c:s")
                .arg("mov_text")
                .arg("-metadata:s:s:0")
                .arg("language=eng");
        }

        // Audio and container options
        command
            .arg("-c:a")
            .arg("aac")
            .arg("-b:a")
            .arg("128k")
            .arg("-profile:v")
            .arg("main")
            .arg("-level:v")
            .arg("4.0")
            .arg("-movflags")
            .arg("+faststart")
            .arg(&output_file);

        // Print for debugging
        println!(
            "🛠️ Running ffmpeg command: ffmpeg {}",
            command
                .get_args()
                .map(|a| a.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        );

        match command.spawn() {
            Ok(mut child) => {
                println!("🚀 PID: {}", child.id());
                match child.wait() {
                    Ok(status) => {
                        if !status.success() {
                            println!("💥 ffmpeg exited with error: {:?}", status);
                            println!("📄 Check log file: {}", log_file.display());
                            return;
                        }
                    }
                    Err(e) => {
                        println!("💥 Failed to wait on ffmpeg: {}", e);
                        return;
                    }
                }

                let duration = start_time.elapsed();
                println!("🏁 Done {} in {:.2?}", base, duration);

                match fs::copy(&output_file, &input_file.with_extension("converted.mp4")) {
                    Ok(_) => {
                        println!(
                            "📝 Copied from {} to {}",
                            output_file.display(),
                            input_file.with_extension("converted.mp4").display()
                        );
                    }
                    Err(e) => {
                        println!("⚠️ Failed to copy converted.mp4: {}", e);
                    }
                }

                append_to_ledger(base);
            }
            Err(e) => {
                println!("💥 Failed to spawn ffmpeg: {}", e);
            }
        }
    });
}

fn collect_files(watch_dir: &str) -> (HashMap<String, PathBuf>, HashMap<String, PathBuf>) {
    let mut mkv_files = HashMap::new();
    let mut srt_files = HashMap::new();

    for entry in walkdir::WalkDir::new(watch_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
    {
        let path = entry.path();
        match path.extension().and_then(|s| s.to_str()) {
            Some("mkv") => {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    mkv_files.insert(stem.to_string(), path.to_path_buf());
                }
            }
            Some("srt") => {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    srt_files.insert(stem.to_string(), path.to_path_buf());
                }
            }
            _ => {}
        }
    }
    (mkv_files, srt_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    #[test]
    fn test_collect_files_finds_mkv_and_srt() {
        let tmp_dir = "/tmp/test_videos";
        let _ = fs::create_dir_all(tmp_dir);

        let mkv_path = format!("{}/video1.mkv", tmp_dir);
        let srt_path = format!("{}/video1.srt", tmp_dir);
        File::create(&mkv_path).unwrap();
        File::create(&srt_path).unwrap();

        let (mkv_map, srt_map) = collect_files(tmp_dir);
        assert!(mkv_map.contains_key("video1"));
        assert!(srt_map.contains_key("video1"));

        let _ = fs::remove_dir_all(tmp_dir);
    }
}
