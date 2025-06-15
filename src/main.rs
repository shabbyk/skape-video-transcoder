fn main() {
    video_transcoder::init_thread_pool(6);
    println!("Starting media watcher on: {}", video_transcoder::WATCH_DIR);

    video_transcoder::start_watcher();
}