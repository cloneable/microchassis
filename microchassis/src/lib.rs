#![deny(unsafe_code, rust_2018_idioms)]
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::panic,
    clippy::unseparated_literal_suffix,
    clippy::unwrap_used,
    // clippy::expect_used, // TODO: revisit
    clippy::unwrap_in_result,
)]
#![allow(
    dead_code, // TODO: remove
    clippy::cargo_common_metadata, // TODO: revisit
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::unnecessary_wraps,
    clippy::use_self,
    clippy::unwrap_in_result, // TODO: revisit
    clippy::multiple_crate_versions,
)]
#![cfg_attr(not(feature = "std"), no_std)]

mod allocator;
pub mod error;
mod jemalloc;
mod logging;
#[cfg(feature = "metrics")]
mod metrics;
mod runtime;
mod signals;
#[cfg(feature = "tracing")]
mod tracing;

use crate::error::ShutdownError;
use error::InitError;
pub use signals::ShutdownBroadcast;

#[inline]
pub fn init() -> Result<ShutdownBroadcast, InitError> {
    logging::init()?;

    let shutdown_signal = signals::init()?;

    #[cfg(feature = "metrics")]
    metrics::init()?;

    #[cfg(feature = "tracing")]
    tracing::init()?;

    runtime::Runtime::new().start()?;

    Ok(shutdown_signal)
}

#[inline]
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
