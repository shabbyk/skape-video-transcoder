use crate::processing::process_directory;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config, EventKind};
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;

pub fn start_watch(watch_dir: &str) {
    println!("Starting media watcher on: {}", watch_dir);

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default()).expect("Failed to initialize watcher");
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
                    _ => {} // Ignore other events
                }
            }
            Ok(Err(e)) => eprintln!("Notify error: {:?}", e),
            Err(e) => eprintln!("Receive error: {:?}", e),
        }
    }
}
