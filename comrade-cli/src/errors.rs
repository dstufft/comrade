use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum TerminalError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub(crate) enum ApplicationError {
    #[error(transparent)]
    TerminalError(#[from] TerminalError),

    #[error(transparent)]
    LogWatcherError(#[from] comrade::errors::LogWatcherError),

    #[error(transparent)]
    ConfigError(#[from] comrade::errors::ConfigError),
}
