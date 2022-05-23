#![warn(clippy::disallowed_types)]

use std::path::PathBuf;
use std::sync::Arc;

use arc_swap::ArcSwap;

mod config;
mod driver;
pub mod errors;
pub mod events;
mod triggers;
mod watcher;

pub mod meta {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

type Result<T, E = errors::ComradeError> = core::result::Result<T, E>;

pub struct Comrade {
    config: config::ConfigRef,
    watchers: watcher::Watchers,
    driver: driver::Driver,
}

impl Default for Comrade {
    fn default() -> Comrade {
        Comrade::new()
    }
}

impl Comrade {
    pub fn new() -> Comrade {
        let config = Arc::new(ArcSwap::from_pointee(config::Config::default()));
        let watchers = watcher::Watchers::default();
        let driver = driver::Driver::create(config.clone(), watchers.receiver());

        Comrade {
            config,
            watchers,
            driver,
        }
    }

    pub fn load(&mut self, config_dir: Option<PathBuf>) -> Result<()> {
        let config = match config_dir {
            Some(path) => Arc::new(config::Config::from_config_dir(path)?),
            None => Arc::new(config::Config::from_default_dir()?),
        };

        self.config.store(config);

        Ok(())
    }

    pub fn init(&mut self) -> Result<()> {
        for (id, c) in self.config().characters.iter() {
            self.watchers.add(id.clone(), c.filename.clone())?;
        }

        self.apply_watcher_filters()?;

        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        self.watchers.start()?;

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.watchers.stop()?;

        Ok(())
    }

    pub fn event(&self) -> Option<events::Event> {
        self.driver.event()
    }
}

impl Comrade {
    fn config(&self) -> config::LoadedConfig {
        self.config.load()
    }

    fn apply_watcher_filters(&mut self) -> Result<()> {
        for id in self.config().characters.keys() {
            // TODO: We need to let you turn these triggers on/off per character.
            self.watchers
                .set_filter(id, self.config().triggers.as_filter()?);
        }

        Ok(())
    }
}
