#![warn(clippy::disallowed_types)]

use std::collections::HashMap;
use std::path::PathBuf;

mod config;
pub mod errors;
mod triggers;
mod watcher;

pub mod meta {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

type Result<T, E = errors::ComradeError> = core::result::Result<T, E>;

pub enum LoadOptions {
    All { config_dir: Option<PathBuf> },
    Config { config_dir: Option<PathBuf> },
    Triggers,
}

#[derive(Default)]
pub struct Comrade {
    config: config::Config,
    triggers: triggers::Triggers,
    watchers: HashMap<String, watcher::LogWatcher>,
}

impl Comrade {
    pub fn new() -> Comrade {
        Comrade::default()
    }

    pub fn load(&mut self, opts: LoadOptions) -> Result<()> {
        match opts {
            LoadOptions::Config { config_dir } => self.load_config(config_dir)?,
            LoadOptions::Triggers => self.load_triggers()?,
            LoadOptions::All { config_dir } => {
                self.load_config(config_dir)?;
                self.load_triggers()?;
            }
        }

        Ok(())
    }

    pub fn init(&mut self) -> Result<()> {
        for (id, c) in self.config.characters.iter() {
            self.watchers
                .insert(id.clone(), watcher::LogWatcher::new(c.filename.clone())?);
        }

        self.apply_watcher_filters()?;

        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        for watcher in self.watchers.values_mut() {
            watcher.start()?;
        }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        for watcher in self.watchers.values_mut() {
            watcher.stop()?;
        }

        Ok(())
    }
}

impl Comrade {
    fn load_config(&mut self, config_dir: Option<PathBuf>) -> Result<()> {
        self.config = match config_dir {
            Some(path) => config::Config::from_config_dir(path)?,
            None => config::Config::from_default_dir()?,
        };

        Ok(())
    }

    fn load_triggers(&mut self) -> Result<()> {
        self.triggers = triggers::Triggers::load(self.config.dirs.data.as_path())?;
        self.apply_watcher_filters()?;

        Ok(())
    }

    fn apply_watcher_filters(&mut self) -> Result<()> {
        for watcher in self.watchers.values() {
            watcher.set_filter(self.triggers.as_filter()?);
        }

        Ok(())
    }
}
