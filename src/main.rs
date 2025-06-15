use rayon::ThreadPoolBuilder;

mod gpu;
mod ledger;
mod processing;
mod watcher;

const WATCH_DIR: &str = "/mnt/smb";

fn main() {
    ThreadPoolBuilder::new().num_threads(6).build_global().unwrap();
    println!("Starting media watcher on: {}", WATCH_DIR);

    watcher::start_watch(WATCH_DIR);
}
