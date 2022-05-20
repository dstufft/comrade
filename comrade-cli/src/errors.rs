use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum InitializationError {
    #[error(transparent)]
    CtrlCHandlerError(#[from] ctrlc::Error),
}
