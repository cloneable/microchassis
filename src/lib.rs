pub mod error;

use error::ChassisError;

pub async fn init() -> Result<(), ChassisError> {
    Ok(())
}
