#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod error;
mod logging;
#[cfg(feature = "metrics")]
mod metrics;
mod signals;
#[cfg(feature = "tracing")]
mod tracing;

use crate::error::ShutdownError;
use error::InitError;
pub use signals::ShutdownBroadcast;

pub fn init() -> Result<ShutdownBroadcast, InitError> {
    logging::init()?;

    let shutdown_signal = signals::init()?;

    #[cfg(feature = "metrics")]
    metrics::init()?;

    #[cfg(feature = "tracing")]
    tracing::init()?;

    Ok(shutdown_signal)
}

pub fn shutdown() -> Result<(), ShutdownError> {
    // TODO: handle errors here?

    #[cfg(feature = "tracing")]
    tracing::shutdown()?;

    #[cfg(feature = "metrics")]
    metrics::shutdown()?;

    signals::shutdown()?;

    logging::shutdown()?;

    Ok(())
}
