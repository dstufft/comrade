use std::time::Duration;

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;

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

    #[clap(required = true)]
    filename: Utf8PathBuf,
}

fn main() -> Result<()> {
    // Parse CLI flags/args
    let cli = Cli::parse();
    let tick_rate = Duration::from_millis(cli.tick_rate);

    // Setup our logger
    tui_logger::init_logger(log::LevelFilter::Trace)?;
    tui_logger::set_default_level(log::LevelFilter::Trace);

    // Setup our terminal
    let mut term = terminal::setup_terminal()?;

    // Actually run our application
    let mut app = App::new("Comrade", cli.filename)?;
    let res = app.run(&mut term, tick_rate);

    // Restore terminal back to it's standard state
    terminal::restore_terminal(term)?;

    // Return our actual error (if there was one), mapped to our Anyhow error
    res.map_err(From::from)
}
