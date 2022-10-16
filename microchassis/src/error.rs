use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum ShutdownError {}
