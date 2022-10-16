use crate::error::{InitError, ShutdownError};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;

pub(crate) fn init() -> Result<(), InitError> {
    // TODO: enable one or the other.
    let prod_logger = tracing_subscriber::fmt::layer()
        .json()
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_current_span(true)
        .with_thread_names(true)
        .with_filter(LevelFilter::INFO);

    let dev_logger = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_filter(LevelFilter::INFO);

    #[cfg(tokio_unstable)]
    {
        // TODO: make console exporter configurable.
        let tokio_console = console_subscriber::spawn();
        tracing_subscriber::registry()
            .with(tokio_console)
            .with(dev_logger)
            .with(prod_logger)
            .init();
    }

    #[cfg(not(tokio_unstable))]
    {
        tracing_subscriber::registry()
            .with(dev_logger)
            .with(prod_logger)
            .init();
    }

    Ok(())
}

pub(crate) fn shutdown() -> Result<(), ShutdownError> {
    // TODO: shut down tracing
    Ok(())
}
