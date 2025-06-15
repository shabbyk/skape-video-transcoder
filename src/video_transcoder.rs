use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use chrono::Local;
use notify::{DebouncedEvent, watcher, RecursiveMode, Watcher};
use walkdir::WalkDir;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

pub const LEDGER_PATH: &str = "/var/tmp/converted_ledger.txt";
pub const TEMP_DIR: &str = "/tmp/video_convert_work";
pub const WATCH_DIR: &str = "/mnt/smb";

pub fn init_thread_pool(threads: usize) {
    ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();
}

pub fn start_watcher() {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(10)).expect("Failed to initialize watcher");
    watcher.watch(WATCH_DIR, RecursiveMode::Recursive).expect("Failed to watch directory");

    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Create(_)) | Ok(DebouncedEvent::Write(_)) => {
                let dir = WATCH_DIR.to_string();
                thread::spawn(move || {
                    process_directory(&dir);
                });
            },
            Err(e) => println!("Watch error: {:?}", e),
            _ => {},
        }
    }
}

fn detect_gpu_type() -> &'static str {
    let output = Command::new("ffmpeg").arg("-encoders").output();
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("h264_nvenc") {
            println!("Detected NVIDIA GPU with NVENC support");
            return "nvenc";
        } else if stdout.contains("h264_vaapi") {
            println!("Detected VAAPI GPU support");
            return "vaapi";
        }
    }
    println!("No hardware GPU encoding detected, defaulting to vaapi");
    "vaapi"
}

fn load_ledger() -> HashSet<String> {
    if let Ok(file) = File::open(LEDGER_PATH) {
        BufReader::new(file).lines().filter_map(Result::ok).collect()
    } else {
        HashSet::new()
    }
}

fn append_to_ledger(entry: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(LEDGER_PATH) {
        let _ = writeln!(file, "{}", entry);
    }
}

fn collect_files(watch_dir: &str) -> (HashMap<String, PathBuf>, HashMap<String, PathBuf>) {
    let mut mkv_files = HashMap::new();
    let mut srt_files = HashMap::new();

    for entry in WalkDir::new(watch_dir).into_iter().filter_map(Result::ok).filter(|e| e.path().is_file()) {
        let path = entry.path();
        match path.extension().and_then(|s| s.to_str()) {
            Some("mkv") => {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    mkv_files.insert(stem.to_string(), path.to_path_buf());
                }
            },
            Some("srt") => {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    srt_files.insert(stem.to_string(), path.to_path_buf());
                }
            },
            _ => {},
        }
    }
    (mkv_files, srt_files)
}

fn process_directory(watch_dir: &str) {
    let (mkv_files, srt_files) = collect_files(watch_dir);
    let ledger = load_ledger();
    let gpu_type = detect_gpu_type();

    fs::create_dir_all(TEMP_DIR).ok();

    mkv_files.par_iter().for_each(|(base, input_file)| {
        if ledger.contains(base) {
            println!("Already converted: {}, skipping...", base);
            return;
        }

        let temp_input = Path::new(TEMP_DIR).join(input_file.file_name().unwrap());
        if let Err(e) = fs::copy(&input_file, &temp_input) {
            println!("Failed to copy to temp: {}", e);
            return;
        }

        let output_file = input_file.with_extension("mp4");
        if output_file.exists() {
            println!("Output already exists: {:?}, skipping...", output_file);
            append_to_ledger(base);
            return;
        }

        let srt_file = srt_files.get(base);
        let log_file = output_file.with_extension("log");

        println!("\n[{}] Processing: {:?}", Local::now().format("%Y-%m-%d %H:%M:%S"), input_file);
        let start_time = std::time::Instant::now();

        let mut command = Command::new("ffmpeg");
        command.arg("-y");

        match gpu_type {
            "nvenc" => {
                command.arg("-hwaccel").arg("cuda")
                       .arg("-i").arg(&temp_input)
                       .arg("-c:v").arg("h264_nvenc");
            },
            _ => {
                command.arg("-hwaccel").arg("vaapi")
                       .arg("-vaapi_device").arg("/dev/dri/renderD128")
                       .arg("-i").arg(&temp_input)
                       .arg("-vf").arg("format=nv12,hwupload")
                       .arg("-c:v").arg("h264_vaapi");
            },
        }

        if let Some(srt_path) = srt_file {
            println!("Subtitle found: {:?}", srt_path);
            command.arg("-i").arg(srt_path)
                   .arg("-c:s").arg("mov_text")
                   .arg("-metadata:s:s:0").arg("language=eng");
        } else {
            println!("No matching subtitle found for: {:?}", input_file);
        }

        command.arg("-c:a").arg("aac")
               .arg("-b:a").arg("128k")
               .arg("-profile:v").arg("main")
               .arg("-level:v").arg("4.0")
               .arg("-movflags").arg("+faststart")
               .arg(&output_file)
               .stdout(Stdio::piped())
               .stderr(File::create(&log_file).expect("Failed to create log file"));

        match command.spawn() {
            Ok(mut child) => {
                println!("Started ffmpeg with PID: {}", child.id());
                let _ = child.wait();
                let duration = start_time.elapsed();
                println!("Finished {} in {:.2?}", base, duration);
                let _ = fs::copy(&output_file, &input_file.with_extension("converted.mp4"));
                append_to_ledger(base);
            },
            Err(e) => println!("Failed to start ffmpeg: {}", e),
        }
    });
}
