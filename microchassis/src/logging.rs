use crate::error::{InitError, ShutdownError};

pub fn init() -> Result<(), InitError> {
    Ok(())
}

pub fn shutdown() -> Result<(), ShutdownError> {
    log::logger().flush();
    Ok(())
}
