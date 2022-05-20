use std::{thread, time};

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};

use comrade::logwatch::LogManager;

mod errors;
mod utils;

#[derive(Debug, Parser)]
#[clap(version)]
struct Cli {
    #[clap(flatten)]
    verbose: Verbosity<WarnLevel>,

    #[clap(required = true)]
    filename: Utf8PathBuf,
}

fn main() -> Result<()> {
    // Parse CLI flags/args
    let cli = Cli::parse();

    // Setup our Ctrl+C handler so that we can exit cleanly
    let running = utils::setup_ctrlc_handler()?;

    let mut manager = LogManager::new()?;
    manager.add(cli.filename)?;

    // Our main loop, currently does nothing but keep the program running
    // until someone hits Ctrl+C
    while utils::should_continue(&running) {
        thread::sleep(time::Duration::from_secs(1))
    }

    Ok(())
}
