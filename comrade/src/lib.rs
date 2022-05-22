use std::collections::HashMap;
use std::path::Path;

mod config;
pub mod errors;
mod parser;
mod watcher;

pub mod meta {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

type Result<T, E = errors::ComradeError> = core::result::Result<T, E>;

pub struct Comrade {
    config: config::Config,
    watchers: HashMap<String, watcher::LogWatcher>,
}

impl Comrade {
    pub fn new() -> Comrade {
        Comrade {
            config: config::Config::default(),
            watchers: HashMap::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, config_dir: Option<P>) -> Result<()> {
        self.config = match config_dir {
            Some(path) => config::Config::from_config_dir(path.as_ref())?,
            None => config::Config::from_default_dir()?,
        };

        Ok(())
    }

    pub fn init(&mut self) -> Result<()> {
        for (id, c) in self.config.characters.iter() {
            self.watchers
                .insert(id.clone(), watcher::LogWatcher::new(c.filename.clone())?);
        }

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

impl Default for Comrade {
    fn default() -> Comrade {
        Comrade::new()
    }
}
