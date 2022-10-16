use crate::error::{InitError, ShutdownError};

pub(crate) fn init() -> Result<(), InitError> {
    Ok(())
}

pub(crate) fn shutdown() -> Result<(), ShutdownError> {
    log::logger().flush();
    Ok(())
}
