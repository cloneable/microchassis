use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChassisError {
    #[error("init error: {0}")]
    InitError(#[from] std::io::Error),
}
