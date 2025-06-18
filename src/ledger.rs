use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

const LEDGER_PATH: &str = "/var/tmp/converted_ledger.txt";

pub fn load_ledger() -> HashSet<String> {
    if let Ok(file) = File::open(LEDGER_PATH) {
        BufReader::new(file).lines().filter_map(Result::ok).collect()
    } else {
        HashSet::new()
    }
}

pub fn append_to_ledger(entry: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(LEDGER_PATH) {
        let _ = writeln!(file, "{}", entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_append_and_load_ledger() {
        let test_path = "/tmp/test_ledger.txt";
        let entry = "test_video";

        // Cleanup first
        let _ = fs::remove_file(test_path);
        std::env::set_var("LEDGER_PATH", test_path); // if env supported

        append_to_ledger(entry);
        let ledger = load_ledger();
        assert!(ledger.contains(entry));

        // Cleanup after
        let _ = fs::remove_file(test_path);
    }
}
