use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arc_swap::{ArcSwap, Cache, Guard};
use platform_dirs::AppDirs;
use serde::Deserialize;

use crate::config::triggers::Triggers;
use crate::errors::ConfigError;
use crate::meta;

pub(crate) mod triggers;

const CONFIG_FILENAME: &str = "Config.toml";

type Result<T, E = ConfigError> = core::result::Result<T, E>;

pub(crate) type ConfigRef = Arc<ArcSwap<Config>>;
pub(crate) type LoadedConfig = Guard<Arc<Config>>;
pub(crate) type CachedConfig = Cache<ConfigRef, Arc<Config>>;

fn default_dirs() -> AppDirs {
    AppDirs::new(Some(meta::PKG_NAME_DISPLAY), false)
        .expect("could not determine application directories")
}

#[derive(Deserialize, Debug)]
pub(crate) struct Directories {
    #[serde(skip)]
    pub(crate) config: PathBuf,
    pub(crate) data: PathBuf,
}

impl Default for Directories {
    fn default() -> Directories {
        let dirs = default_dirs();

        Directories {
            config: dirs.config_dir,
            data: dirs.data_dir,
        }
    }
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq, Hash, Clone)]
#[serde(transparent)]
pub(crate) struct CharacterId(String);

#[derive(Deserialize, Debug)]
pub(crate) struct Character {
    #[serde(rename = "name")]
    pub(crate) _name: String,
    #[serde(rename = "server")]
    pub(crate) _server: String,
    pub(crate) filename: PathBuf,
}

#[derive(Deserialize, Debug, Default)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) dirs: Directories,

    #[serde(default)]
    pub(crate) characters: HashMap<CharacterId, Character>,

    #[serde(skip)]
    pub(crate) triggers: Triggers,
}

impl Config {
    pub(crate) fn from_default_dir() -> Result<Config> {
        let filename = default_dirs().config_dir.join(CONFIG_FILENAME);
        match try_open_config_file(filename.as_path(), true)? {
            Some(file) => parse_config(filename.as_path(), file),
            None => Ok(Config::default()),
        }
    }

    pub(crate) fn from_config_dir(path: PathBuf) -> Result<Config> {
        let filename = path.join(CONFIG_FILENAME);
        let file = try_open_config_file(filename.as_path(), false)?
            .expect("None from try_open_config_file with allow_missing=false?");
        let mut config = parse_config(filename.as_path(), file)?;

        config.dirs.config = path;
        config.triggers = Triggers::load(config.dirs.data.as_path())?;

        Ok(config)
    }
}

fn parse_config(filename: &Path, mut file: fs::File) -> Result<Config> {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    toml_edit::de::from_str(buffer.as_str()).map_err(|source| ConfigError::DeserializationError {
        source,
        filename: filename.to_path_buf(),
    })
}

fn try_open_config_file(filename: &Path, allow_missing: bool) -> Result<Option<fs::File>> {
    let file = fs::OpenOptions::new().read(true).open(filename);

    let file = match file {
        Ok(f) => Ok(Some(f)),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound && allow_missing {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }?;

    Ok(file)
}
