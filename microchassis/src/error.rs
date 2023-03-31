use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}
