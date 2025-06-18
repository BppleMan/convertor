use crate::boslife::boslife_service::BosLifeService;
use crate::error::AppError;
use color_eyre::eyre::eyre;
use color_eyre::Result;

pub mod clash;
pub mod surge;
pub mod subscription;

pub struct AppState {
    pub service: BosLifeService,
}

pub async fn root() -> Result<(), AppError> {
    Err(eyre!("Hello, World!"))?;
    Ok(())
}
