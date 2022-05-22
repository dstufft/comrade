use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use platform_dirs::AppDirs;
use serde::Deserialize;

use crate::errors::ConfigError;
use crate::meta;

type Result<T, E = ConfigError> = core::result::Result<T, E>;

const CONFIG_FILENAME: &str = "Config.toml";

fn default_dirs() -> AppDirs {
    AppDirs::new(Some(meta::PKG_NAME_DISPLAY), false)
        .expect("could not determine application directories")
}

#[derive(Deserialize, Debug)]
pub struct Directories {
    #[serde(skip)]
    pub config: PathBuf,
}

impl Default for Directories {
    fn default() -> Directories {
        let dirs = default_dirs();

        Directories {
            config: dirs.config_dir,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Character {
    pub name: String,
    pub server: String,
    pub filename: PathBuf,
}

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub dirs: Directories,

    #[serde(default)]
    pub characters: HashMap<String, Character>,
}

impl Config {
    pub fn from_default_dir() -> Result<Config> {
        match try_open_config_file(default_dirs().config_dir, true)? {
            Some(file) => parse_config(file),
            None => Ok(Config {
                dirs: Directories::default(),
                characters: HashMap::new(),
            }),
        }
    }

    pub fn from_config_dir<P: AsRef<Path>>(path: P) -> Result<Config> {
        let file = try_open_config_file(&path, false)?.unwrap();
        let mut config = parse_config(file)?;

        config.dirs.config = path.as_ref().to_path_buf();

        Ok(config)
    }
}

fn parse_config(mut file: fs::File) -> Result<Config> {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(toml_edit::de::from_str(buffer.as_str())?)
}

fn try_open_config_file<P: AsRef<Path>>(path: P, allow_missing: bool) -> Result<Option<fs::File>> {
    let path = path.as_ref();
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
