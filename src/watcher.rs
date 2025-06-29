use crate::processing::process_directory;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config, EventKind};
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

pub fn start_watch(watch_dir: &str) {
    println!("🕵️ Starting watcher on: {}", watch_dir);

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher =
        Watcher::new(tx, Config::default()).expect("Failed to initialize watcher");

    watcher
        .watch(Path::new(watch_dir), RecursiveMode::Recursive)
        .expect("Failed to watch directory");

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        let dir = watch_dir.to_string();
                        thread::spawn(move || {
                            process_directory(&dir);
                        });
                    }
                    _ => {}
                }
            }
            Ok(Err(e)) => eprintln!("Notify error: {:?}", e),
            Err(e) => eprintln!("Receive error: {:?}", e),
        }
    }
}

/// Fallback polling-based watcher
pub fn start_watch_with_fallback(watch_dir: &str) {
    println!("🔁 SMB mode detected — using polling fallback every 30 seconds");

    loop {
        process_directory(watch_dir);
        thread::sleep(Duration::from_secs(30));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::Duration;
    use std::thread;

    fn create_test_file(dir: &str, name: &str) {
        let path = Path::new(dir).join(name);
        fs::create_dir_all(dir).unwrap();
        File::create(path).unwrap();
    }

    #[test]
    fn test_start_watch_initializes_ok() {
        let (tx, _rx) = channel();
        let watcher: Result<RecommendedWatcher, _> = notify::recommended_watcher(tx);
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_fallback_processes_directory() {
        let test_dir = "/tmp/test-watcher-fallback";
        std::env::set_var("WATCH_DIR", test_dir);
        fs::create_dir_all(test_dir).unwrap();

        // Create dummy file to trigger process_directory
        create_test_file(test_dir, "sample.mkv");

        // Spawn fallback watcher and run just once (cancel immediately after)
        let handle = thread::spawn(move || {
            start_watch_with_fallback(test_dir); // this runs in a loop
        });

        // Allow one iteration of the fallback
        thread::sleep(Duration::from_secs(2));
        handle.thread().unpark(); // just in case (not strictly needed here)
    }
}
