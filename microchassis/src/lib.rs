#![cfg_attr(not(feature = "std"), no_std)]

pub mod error;
mod logging;
pub mod signals;
#[cfg(feature = "tracing")]
mod tracing;

use crate::error::ShutdownError;
use error::InitError;
use signals::ShutdownSignal;

pub fn init() -> Result<ShutdownSignal, InitError> {
    logging::init()?;

    let shutdown_signal = signals::init()?;

    #[cfg(feature = "tracing")]
    tracing::init()?;

    Ok(shutdown_signal)
}

pub fn shutdown() -> Result<(), ShutdownError> {
    // TODO: handle errors here?
    #[cfg(feature = "tracing")]
    tracing::shutdown()?;

    signals::shutdown()?;

    logging::shutdown()?;

    Ok(())
}
