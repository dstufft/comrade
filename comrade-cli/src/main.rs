use anyhow::Result;
use comrade::logwatch::LogManager;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

const LOGPATH: &str = r"logfile";

fn main() -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut manager = LogManager::new()?;
    manager.add(LOGPATH)?;

    while running.load(Ordering::SeqCst) {
        thread::sleep(time::Duration::from_secs(1))
    }

    Ok(())
}
