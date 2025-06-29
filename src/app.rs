use rayon::ThreadPoolBuilder;

use crate::watcher;
use crate::config;

pub fn start_transcoding_app() {
    let cfg = config::load_config();
    ThreadPoolBuilder::new()
        .num_threads(cfg.threads)
        .build_global()
        .unwrap();

    println!("🎯 Watching directory: {}", cfg.watch_dir);
    println!("📡 SMB mode: {}", cfg.is_smb);
    println!("🧵 Using {} threads", cfg.threads);

    if cfg.is_smb {
        watcher::start_watch_with_fallback(&cfg.watch_dir);
    } else {
        watcher::start_watch(&cfg.watch_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_start_transcoding_app_config_parsing() {
        // Set test environment variables
        env::set_var("WATCH_DIR", "/tmp/test-dir");
        env::set_var("IS_SMB", "true");
        env::set_var("THREADS", "3");

        // Load config and validate
        let cfg = config::load_config();

        assert_eq!(cfg.watch_dir, "/tmp/test-dir");
        assert!(cfg.is_smb);
        assert_eq!(cfg.threads, 3);

        // Clean up env vars
        env::remove_var("WATCH_DIR");
        env::remove_var("IS_SMB");
        env::remove_var("THREADS");
    }
}
