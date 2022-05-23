use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogWatcherError {
    #[error("could not create file notifier")]
    FileNotifierError(#[from] notify::Error),

    #[error("already watching {path:?}")]
    AlreadyWatching { path: PathBuf },

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("invalid file path")]
    InvalidPath { path: PathBuf },
}

#[derive(Error, Debug)]
pub enum DriverError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("could not parse configuration")]
    DeserializationError {
        source: toml_edit::de::Error,
        filename: PathBuf,
    },
}

#[derive(Error, Debug)]
pub enum TriggerError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("could not parse triggers")]
    DeserializationError {
        source: toml_edit::de::Error,
        filename: PathBuf,
    },

    #[error("invalid regex")]
    InvalidRegex(#[from] regex::Error),
}

#[derive(Error, Debug)]
pub enum ComradeError {
    #[error(transparent)]
    ConfigError(#[from] ConfigError),

    #[error(transparent)]
    TriggerError(#[from] TriggerError),

    #[error(transparent)]
    LogWatcherError(#[from] LogWatcherError),
}
