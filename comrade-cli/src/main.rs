use comrade::logwatch::LogManager;
use std::{thread, time};

const LOGPATH: &str = r"logfile";

fn main() {
    let mut manager = LogManager::new();
    manager.add(LOGPATH);

    loop {
        thread::sleep(time::Duration::from_secs(1))
    }
}
