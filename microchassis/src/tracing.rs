use crate::error::{InitError, ShutdownError};

pub(crate) fn init() -> Result<(), InitError> {
    #[cfg(tokio_unstable)]
    console_subscriber::init();

    Ok(())
}

pub(crate) fn shutdown() -> Result<(), ShutdownError> {
    Ok(())
}
