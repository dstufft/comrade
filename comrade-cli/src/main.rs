#![warn(clippy::disallowed_types)]

use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use path_clean::PathClean;

use comrade::meta;
use comrade::{Comrade, LoadOptions};

use crate::app::App;

mod app;
mod errors;
mod terminal;
mod ui;

#[derive(Debug, Parser)]
#[clap(version)]
struct Cli {
    #[clap(long, default_value_t = 250)]
    tick_rate: u64,

    #[clap(long)]
    config_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    // Parse CLI flags/args
    let cli = Cli::parse();

    // Setup our logger
    tui_logger::init_logger(log::LevelFilter::Trace)?;
    tui_logger::set_default_level(log::LevelFilter::Trace);

    // Setup our terminal
    let mut term = terminal::setup_terminal()?;

    // Run our application, this is done inside of a function so that
    // we can use ? without returning early, in effect we've created
    // a psuedo try ... finally block.
    let res = (|| -> Result<()> {
        let tick_rate = Duration::from_millis(cli.tick_rate);

        // Get our configuration directory
        let config_dir = match cli.config_dir {
            Some(path) => Some(absolute_path(path)?),
            None => None,
        };

        // Setup Comrade
        let mut comrade = Comrade::new();
        comrade
            .load(LoadOptions::Config {
                config_dir: config_dir.clone(),
            })
            .with_context(|| format!("failed to read configuration from {:?}", config_dir))?;
        comrade
            .load(LoadOptions::Triggers)
            .context("failed to load triggers")?;

        // Actually run our application
        let mut app = App::new(meta::PKG_NAME_DISPLAY, comrade);
        let res = app.run(&mut term, tick_rate);

        res.map_err(From::from)
    })();

    // Restore terminal back to it's standard state
    terminal::restore_terminal(term)?;

    // Return our actual error (if there was one), mapped to our Anyhow error
    res.map_err(From::from)
}

fn absolute_path<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let path = path.as_ref();
    let abspath = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    }
    .clean();

    Ok(abspath)
}
