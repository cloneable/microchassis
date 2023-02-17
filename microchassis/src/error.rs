use log::SetLoggerError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("logging error: {0}")]
    LoggingError(#[from] SetLoggerError),

    #[error("runtime error: {0}")]
    RuntimeError(#[from] RuntimeError),
}

#[derive(Error, Debug)]
pub enum ShutdownError {}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}
