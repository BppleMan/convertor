use crate::error::AppError;
use color_eyre::eyre::eyre;
use color_eyre::Result;

pub mod clash;
pub mod surge;

pub async fn root() -> Result<(), AppError> {
    Err(eyre!("Hello, World!"))?;
    Ok(())
}
