use crate::error::{InitError, ShutdownError};

pub(crate) fn init() -> Result<(), InitError> {
    tracing_log::LogTracer::builder()
        .with_max_level(log::LevelFilter::Debug)
        .init()?;
    Ok(())
}

pub(crate) fn shutdown() -> Result<(), ShutdownError> {
    log::logger().flush();
    Ok(())
}
