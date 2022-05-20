use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::errors::InitializationError;

type InitializationResult<T, E = InitializationError> = core::result::Result<T, E>;

pub(crate) fn setup_ctrlc_handler() -> InitializationResult<Arc<AtomicBool>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    Ok(running)
}

pub(crate) fn should_continue(running: &Arc<AtomicBool>) -> bool {
    running.load(Ordering::SeqCst)
}
