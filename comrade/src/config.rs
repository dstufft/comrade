use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use platform_dirs::AppDirs;
use serde::Deserialize;

use crate::errors::ConfigError;
use crate::meta;

const CONFIG_FILENAME: &str = "Config.toml";

type Result<T, E = ConfigError> = core::result::Result<T, E>;

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
    pub(crate) characters: HashMap<String, Character>,
}

impl Config {
    pub(crate) fn from_default_dir() -> Result<Config> {
        match try_open_config_file(default_dirs().config_dir.as_path(), true)? {
            Some(file) => parse_config(file),
            None => Ok(Config {
                dirs: Directories::default(),
                characters: HashMap::new(),
            }),
        }
    }

    pub(crate) fn from_config_dir(path: PathBuf) -> Result<Config> {
        let file = try_open_config_file(path.as_path(), false)?
            .expect("None from try_open_config_file with allow_missing=false?");
        let mut config = parse_config(file)?;

        config.dirs.config = path;

        Ok(config)
    }
}

fn parse_config(mut file: fs::File) -> Result<Config> {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(toml_edit::de::from_str(buffer.as_str())?)
}

fn try_open_config_file(path: &Path, allow_missing: bool) -> Result<Option<fs::File>> {
    let file = fs::OpenOptions::new()
        .read(true)
        .open(path.join(CONFIG_FILENAME));

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
