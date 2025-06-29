use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Helper to create dummy MKV using FFmpeg lavfi testsrc.
fn create_dummy_mkv(path: &PathBuf) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create MKV parent directory");
    }

    let output = std::process::Command::new("ffmpeg")
        .args(&[
            "-f", "lavfi",
            "-i", "testsrc=duration=5:size=128x128:rate=15",
            "-pix_fmt", "yuv420p",
            "-c:v", "libx264",
            "-t", "5",
            "-y",
            path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute ffmpeg");

    if !output.status.success() {
        eprintln!("FFmpeg stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("FFmpeg stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("FFmpeg failed to generate dummy MKV");
    }

    assert!(path.exists(), "Expected MKV file was not created");
}

/// Creates a dummy SRT subtitle file with one entry.
fn create_dummy_srt(path: &PathBuf) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create SRT parent directory");
    }

    let mut file = File::create(path).expect("Failed to create dummy SRT");
    writeln!(file, "1\n00:00:00,000 --> 00:00:01,000\nHello, world!").unwrap();
}

#[test]
fn test_process_directory_creates_mp4() {
    std::env::set_var("FORCE_CPU", "1");

    let test_dir = std::fs::canonicalize("test-files")
        .or_else(|_| {
            std::fs::create_dir_all("test-files")?;
            std::fs::canonicalize("test-files")
        })
        .expect("Failed to create or resolve test dir");

    let mkv_path = test_dir.join("sample.mkv");
    let srt_path = test_dir.join("sample.srt");
    let expected_output = test_dir.join("sample.mp4");
    let converted_output = test_dir.join("sample.converted.mp4");
    let log_path = test_dir.join("sample.log");

    create_dummy_mkv(&mkv_path);
    create_dummy_srt(&srt_path);

    assert!(mkv_path.exists(), "MKV was not created");
    assert!(srt_path.exists(), "SRT was not created");

    video_transcoder::processing::process_directory(test_dir.to_str().unwrap());

    let start = Instant::now();
    let timeout = Duration::from_secs(20);
    while !(expected_output.exists() && converted_output.exists()) && start.elapsed() < timeout {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    if !expected_output.exists() {
        eprintln!("❌ MP4 file was not created.");
        if log_path.exists() {
            eprintln!("---- FFmpeg Log ----");
            let log = std::fs::read_to_string(&log_path).unwrap_or_default();
            eprintln!("{}", log);
        }
    }

    assert!(
        expected_output.exists(),
        "MP4 was not created at expected location"
    );

    let _ = std::fs::remove_dir_all(&test_dir);
}
