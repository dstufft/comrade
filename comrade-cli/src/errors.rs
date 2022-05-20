use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum TerminalError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub(crate) enum ApplicationError {
    #[error(transparent)]
    TerminalError(#[from] TerminalError),

    #[error(transparent)]
    LogWatcherError(#[from] comrade::errors::LogWatcherError),
}
