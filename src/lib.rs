#![cfg_attr(not(feature = "std"), no_std)]

pub mod error;

use error::ChassisError;

pub async fn init() -> Result<(), ChassisError> {
    Ok(())
}
