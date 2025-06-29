pub struct AppConfig {
    pub watch_dir: String,
    pub is_smb: bool,
    pub threads: usize,
}

pub fn load_config() -> AppConfig {
    let watch_dir = std::env::var("WATCH_DIR").unwrap_or_else(|_| "/mnt/smb/Test-transcoding".into());
    let is_smb = std::env::var("IS_SMB")
        .unwrap_or_else(|_| "false".into())
        .to_lowercase() == "true";

    let threads = std::env::var("THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| std::cmp::max(1, num_cpus::get() / 2));

    AppConfig { watch_dir, is_smb, threads }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_load_config_defaults() {
        // Clear env vars
        env::remove_var("WATCH_DIR");
        env::remove_var("IS_SMB");
        env::remove_var("THREADS");

        let cfg = load_config();

        assert_eq!(cfg.watch_dir, "/mnt/smb");
        assert_eq!(cfg.is_smb, false);
        assert!(cfg.threads >= 1);
    }

    #[test]
    fn test_load_config_with_env_vars() {
        env::set_var("WATCH_DIR", "/custom/dir");
        env::set_var("IS_SMB", "true");
        env::set_var("THREADS", "4");

        let cfg = load_config();

        assert_eq!(cfg.watch_dir, "/custom/dir");
        assert_eq!(cfg.is_smb, true);
        assert_eq!(cfg.threads, 4);

        // Clean up to avoid affecting other tests
        env::remove_var("WATCH_DIR");
        env::remove_var("IS_SMB");
        env::remove_var("THREADS");
    }

    #[test]
    fn test_threads_fallback_on_invalid_value() {
        env::set_var("THREADS", "not_a_number");

        let cfg = load_config();
        assert!(cfg.threads >= 1); // fallback to CPU/2

        env::remove_var("THREADS");
    }

    #[test]
    fn test_is_smb_case_insensitive() {
        env::set_var("IS_SMB", "TrUe");
        let cfg = load_config();
        assert!(cfg.is_smb);

        env::set_var("IS_SMB", "FALSE");
        let cfg = load_config();
        assert!(!cfg.is_smb);

        env::remove_var("IS_SMB");
    }
}
