#[test]
fn detect_gpu_type_returns_valid_value() {
    let result = video_transcoder::gpu::detect_gpu_type();
    assert!(
        ["nvenc", "vaapi", "cpu"].contains(&result),
        "Unexpected GPU type: {}",
        result
    );
}

#[test]
fn detect_gpu_type_respects_force_cpu() {
    std::env::set_var("FORCE_CPU", "1");

    let result = video_transcoder::gpu::detect_gpu_type();
    assert_eq!(result, "cpu", "FORCE_CPU was set, but result was {}", result);

    std::env::remove_var("FORCE_CPU");
}
