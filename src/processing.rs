use crate::gpu::detect_gpu_type;
use crate::ledger::{append_to_ledger, load_ledger};
use chrono::Local;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const TEMP_DIR: &str = "/tmp/video_convert_work";

pub fn process_directory(watch_dir: &str) {
    let (mkv_files, srt_files) = collect_files(watch_dir);
    let ledger = load_ledger();
    let gpu_type = detect_gpu_type();

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

        match gpu_type {
            "nvenc" => {
                command
                    .arg("-hwaccel")
                    .arg("cuda")
                    .arg("-i")
                    .arg(&temp_input)
                    .arg("-c:v")
                    .arg("h264_nvenc");
            }
            "vaapi" => {
                command
                    .arg("-hwaccel")
                    .arg("vaapi")
                    .arg("-vaapi_device")
                    .arg("/dev/dri/renderD128")
                    .arg("-i")
                    .arg(&temp_input)
                    .arg("-vf")
                    .arg("format=nv12,hwupload")
                    .arg("-c:v")
                    .arg("h264_vaapi");
            }
            _ => {
                // Fallback for test or headless environments
                println!("⚠️ GPU not available or unsupported, falling back to CPU encoding.");
                command
                    .arg("-i")
                    .arg(&temp_input)
                    .arg("-c:v")
                    .arg("libx264");
            }
        }

        if let Some(srt_path) = srt_file {
            println!("💬 Subtitle found: {:?}", srt_path);
            command
                .arg("-i")
                .arg(srt_path)
                .arg("-c:s")
                .arg("mov_text")
                .arg("-metadata:s:s:0")
                .arg("language=eng");
        } else {
            println!("🕳️ No subtitle found for: {:?}", input_file);
        }

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
            .arg(&output_file)
            .stdout(Stdio::piped())
            .stderr(File::create(&log_file).expect("Failed to create log"));

        match command.spawn() {
            Ok(mut child) => {
                println!("🚀 PID: {}", child.id());
                let _ = child.wait();
                let duration = start_time.elapsed();
                println!("🏁 Done {} in {:.2?}", base, duration);
                let _ = fs::copy(&output_file, &input_file.with_extension("converted.mp4"));
                println!("Copied to {} from {}", output_file.display(), input_file.with_extension("converted.mp4").display());
                append_to_ledger(base);
            }
            Err(e) => println!("💥 Failed: {}", e),
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
