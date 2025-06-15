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
