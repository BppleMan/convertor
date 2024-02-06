use crate::error::AppError;
use anyhow::anyhow;

pub mod clash;
pub mod surge;

pub async fn root() -> Result<(), AppError> {
    Err(anyhow!("Hello, World!"))?;
    Ok(())
}
