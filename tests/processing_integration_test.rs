use std::fs::File;
use std::io::Write;
use std::path::PathBuf;


/// Helper to create dummy MKV using FFmpeg lavfi testsrc.
fn create_dummy_mkv(path: &PathBuf) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create MKV parent directory");
    }

    let output = std::process::Command::new("ffmpeg")
        .args(&[
            "-hwaccel", "none",
            "-t", "5",
            "-s", "128x128",
            "-f", "rawvideo",
            "-pix_fmt", "rgb24",
            "-r", "15",
            "-i", "/dev/zero",
            "-c:v", "libx264",
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
    let mut file = File::create(path).expect("Failed to create dummy SRT");
    writeln!(
        file,
        "1\n00:00:00,000 --> 00:00:01,000\nHello, world!"
    )
    .unwrap();
}

#[test]
fn test_process_directory_creates_mp4() {
    std::env::set_var("FORCE_CPU", "1");

    let test_dir = PathBuf::from("test-files");
    let _ = std::fs::remove_dir_all(&test_dir); // Clean before test
    std::fs::create_dir_all(&test_dir).unwrap();

    let mkv_path = test_dir.join("sample.mkv");
    let srt_path = test_dir.join("sample.srt");
    let expected_output = test_dir.join("sample.mp4");

    create_dummy_mkv(&mkv_path);
    create_dummy_srt(&srt_path);

    assert!(mkv_path.exists(), "MKV was not created");

    // Run the processor
    video_transcoder::processing::process_directory(test_dir.to_str().unwrap());

    std::thread::sleep(std::time::Duration::from_secs(2));

    assert!(
        expected_output.exists(),
        "MP4 was not created at expected location"
    );

    // Clean up after test
    let _ = std::fs::remove_dir_all(&test_dir);
}
