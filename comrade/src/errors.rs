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
